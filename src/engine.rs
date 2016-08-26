use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;
use std::slice::Iter;

use glium::backend::glutin_backend::GlutinFacade;
use glium::{Program, Display, Frame, Surface, VertexBuffer, IndexBuffer};
use glium::glutin::{Window, VirtualKeyCode};
use glium::glutin::Event;
use glium::backend::glutin_backend::PollEventsIter;
use glium::index::IndicesSource;
use glium::vertex::MultiVerticesSource;
use glium::uniforms::Uniforms;
use glium::draw_parameters::PolygonMode;

use time::Duration;

use na::{Point3, Vector3, Matrix3, Matrix4, PerspectiveMatrix3, Rotation3, Norm, ToHomogeneous, Cross};

use gl_util::{Camera, Vertex, SimpleCamera};
use block::{BlockRenderData, Chunk, CHUNK_SIZE, CuboidRegion};

pub struct Game {
	state: Box<GameState>,
	running: bool,
	services: GameServices,
}

pub struct GameServices {
	pub draw_service: DrawService,
	pub input_service: InputService,
}

impl Game {
	pub fn new(start_state: Box<GameState>, display: Display, shaders: Program) -> Game {
		let disp = Rc::new(display);
		Game {
			state: start_state,
			services: GameServices {
				draw_service: DrawService::new(disp.clone(), shaders),
				input_service: InputService::new(disp),
			},
			running: true,
		}
	}
	pub fn is_running(&self) -> bool { self.running }
	pub fn quit(&mut self) -> () { self.running = false }

	fn swap_state(&mut self, state: Box<GameState>) -> Box<GameState> {
		self.state.leaving();
		let old_state = ::std::mem::replace(&mut self.state, state);
		self.state.entered();
		old_state
	}

	pub fn update(&mut self, time_elapsed: &Duration) -> () {
		self.services.input_service.flush_event_queue();
		let result = self.state.update(&self.services, time_elapsed);
		match result {
			UpdateResult::ChangeState(new_state) => { self.swap_state(new_state); },
			UpdateResult::Quit => self.quit(),
			UpdateResult::None => (),
		};
	}

	pub fn draw(&mut self) {
		self.state.draw(&mut self.services.draw_service);
		self.services.draw_service.flush();
	}
}

pub struct InputService {
	display: Rc<Display>,
	events: Vec<Event>,
}

impl InputService {
	pub fn new(display: Rc<Display>) -> InputService {
		let mut ret = InputService {
			display: display,
			events: Vec::new(),
		};
		ret.flush_event_queue();
		ret
	}

	pub fn flush_event_queue(&mut self) {
		self.events = self.display.poll_events().collect();
	}

	pub fn events(&self) -> Iter<Event> {
		self.events.iter()
	}

	pub fn size(&self) -> Option<(u32, u32)> {
		self.display.get_window().and_then(|win| win.get_inner_size_points())
	}

	pub fn set_cursor_position(&self, x: i32, y: i32) {
		match self.display.get_window() {
			Some(win) => { win.set_cursor_position(x, y); },
			None => (),
		}
	}
}

pub struct DrawService {
	display: Rc<Display>,
	frame: Frame,
	program: Program,
	perspective: PerspectiveMatrix3<f32>,
}

impl Drop for DrawService {
	fn drop(&mut self) {
		self.frame.set_finish();
	}
}

impl DrawService {
	fn build_perspective(frame: &Frame) -> PerspectiveMatrix3<f32> {
		let (width, height) = frame.get_dimensions();

		let fov: f32 = ::std::f32::consts::PI / 3.0;
		let zfar = 1024.0;
		let znear = 0.001;

		PerspectiveMatrix3::new(width as f32 / height as f32, fov, znear, zfar)
	}

	pub fn new(display: Rc<Display>, program: Program) -> DrawService {
		let mut frame = display.draw();
		frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
		let perspective = DrawService::build_perspective(&frame);
		DrawService {
			display: display,
			program: program,
			frame: frame,
			perspective: perspective,
		}
	}

	// TODO: Switch to trait object when glium updates.
	pub fn facade(&self) -> &Display {
		&self.display
	}

	pub fn update_perspective(&mut self) {
		self.perspective = DrawService::build_perspective(&self.frame);
	}

	pub fn flush(&mut self) {
		// TODO: Update framerate
		self.frame.set_finish();

		self.frame = self.display.draw();
		self.frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
	}

	pub fn draw_buffer<'a, 'b, I, V>(&mut self, model_view: &Matrix4<f32>, vertices: V, indices: I)
			where I: Into<IndicesSource<'a>>, V: MultiVerticesSource<'b> {
		let uniforms = uniform! {
			u_light: [0.0, 0.0, 1.0f32],
			model_view: model_view.as_ref().clone(),
			perspective: self.perspective.as_matrix().as_ref().clone(),
		};

		use glium::{DrawParameters, Depth};
		use glium::draw_parameters::{DepthTest, BackfaceCullingMode};
		let params = DrawParameters {
			depth: Depth {
				test: DepthTest::IfLess,
				write: true,
				.. Default::default()
			},
			backface_culling: BackfaceCullingMode::CullClockwise,
//			polygon_mode: PolygonMode::Line,
			.. Default::default()
		};

		self.frame.draw(vertices, indices, &self.program, &uniforms, &params).unwrap();
	}
}

pub enum UpdateResult {
	None,
	Quit,
	ChangeState(Box<GameState>),
}

pub trait GameState {
	fn entered(&mut self) -> ();
	fn leaving(&mut self) -> ();

	fn update(&mut self, services: &GameServices, time_elapsed: &Duration) -> UpdateResult;
	fn draw(&self, draw_service: &mut DrawService) -> ();
}

pub struct StatePlaying {
	world: World,
	block_render_types: Vec<BlockRenderData>,
	camera: SimpleCamera<f32>,
	keys_down: HashSet<VirtualKeyCode>,
	region: CuboidRegion,
}

const MOUSE_SENSITIVITY:  f32 = 0.001;
const MOTION_SENSITIVITY: f32 = 0.01;
const MOTION_SENSITIVITY_FAST: f32 = 0.1;

use block::World;
impl StatePlaying {
	pub fn new() -> StatePlaying {
		let world = World::new();
		let region = CuboidRegion::new(&world, -5, -5, -5, 5, 5, 5);
		let mut ret = StatePlaying {
			world: World::new(),
			block_render_types: Vec::with_capacity(2),
			camera: SimpleCamera {
				position:   Point3::new( 0.0,   0.0,  50.0),
				direction: Vector3::new(-0.5,  -0.5,  -4.0).normalize(),
				up:        Vector3::new( 0.0,   0.0,   1.0),
			},
			keys_down: HashSet::new(),
			region: region,
		};
		ret.block_render_types.push(BlockRenderData {
			obscures: 0,
			color: [0.0f32; 3],
			should_render: false,
		});
		ret.block_render_types.push(BlockRenderData {
			obscures: 0b111111,
			color: [0.3, 0.4, 0.2],
			should_render: true,
		});
		ret
	}
}

impl GameState for StatePlaying {
	fn entered(&mut self) -> () {}
	fn leaving(&mut self) -> () {}

	fn update(&mut self, services: &GameServices, time_elapsed: &Duration) -> UpdateResult {
		for ev in services.input_service.events() {
			use glium::glutin::ElementState;
			match ev {
				&Event::Closed => return UpdateResult::Quit,   // the window has been closed by the user
				&Event::KeyboardInput(pressed, _, key) => match key {
					None => (),
					Some(key) => match key {
						VirtualKeyCode::Escape => return UpdateResult::Quit,
						code => match pressed {
							ElementState::Pressed => { self.keys_down.insert(code); },
							ElementState::Released => { self.keys_down.remove(&code); },
						},
					}
				},
				&Event::MouseMoved(raw_x, raw_y) => {
					let (size_x, size_y) = services.input_service.size().unwrap();
					services.input_service.set_cursor_position((size_x / 2) as i32, (size_y / 2) as i32);
					let (delta_x, delta_y) = (raw_x - size_x as i32, raw_y - size_y as i32);
					self.camera.direction *= Rotation3::new(self.camera.up * (delta_x as f32) * MOUSE_SENSITIVITY);
					self.camera.direction *= Rotation3::new(self.camera.up.cross(&self.camera.direction) * (delta_y as f32) * MOUSE_SENSITIVITY);
				},

				_ => ()
			}
		}

		let dolly_speed = if self.keys_down.contains(&VirtualKeyCode::LShift) || self.keys_down.contains(&VirtualKeyCode::RShift) {
			MOTION_SENSITIVITY_FAST
		} else {
			MOTION_SENSITIVITY
		};

		match (self.keys_down.contains(&VirtualKeyCode::A), self.keys_down.contains(&VirtualKeyCode::D)) {
			(true, true) => (),
			(false, false) => (),

			(true, false) => {
				self.camera.position -= self.camera.direction.cross(&self.camera.up) * time_elapsed.num_milliseconds() as f32 * dolly_speed;
			},
			(false, true) => {
				self.camera.position -= -1.0 * self.camera.direction.cross(&self.camera.up) * time_elapsed.num_milliseconds() as f32 * dolly_speed;
			},
		}

		match (self.keys_down.contains(&VirtualKeyCode::W), self.keys_down.contains(&VirtualKeyCode::S)) {
			(true, true) => (),
			(false, false) => (),

			(true, false) => {
				self.camera.position -= -1.0 * self.camera.direction * time_elapsed.num_milliseconds() as f32 * dolly_speed;
			},
			(false, true) => {
				self.camera.position -= self.camera.direction * time_elapsed.num_milliseconds() as f32 * dolly_speed;
			},
		}

		match (self.keys_down.contains(&VirtualKeyCode::E), self.keys_down.contains(&VirtualKeyCode::Q)) {
			(true, true) => (),
			(false, false) => (),

			(true, false) => {
				self.camera.position -= -1.0 * self.camera.up * time_elapsed.num_milliseconds() as f32 * dolly_speed;

			},
			(false, true) => {
				self.camera.position -= self.camera.up * time_elapsed.num_milliseconds() as f32 * dolly_speed;
			},
		}

		UpdateResult::None
	}

	fn draw(&self, draw_service: &mut DrawService) {
		self.region.draw(&self.block_render_types, draw_service, self.camera.to_isometry().to_homogeneous());
	}
}

