use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;
use std::slice::Iter;
use std::mem::replace;

use glium::{Program, Display, Frame, Surface, VertexBuffer, IndexBuffer};
use glium::glutin::{Window, VirtualKeyCode};
use glium::glutin::{EventsLoop, Event, WindowEvent};
use glium::index::IndicesSource;
use glium::vertex::MultiVerticesSource;
use glium::uniforms::Uniforms;
use glium::draw_parameters::PolygonMode;

use time::Duration;

use na::{Point3, Vector3, Matrix3, Matrix4, Perspective3, Rotation3};

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
	pub fn new(start_state: Box<GameState>, display: Display, ev_loop: EventsLoop, shaders: Program)
			-> Game {
		let disp = Rc::new(display);
		Game {
			state: start_state,
			services: GameServices {
				draw_service: DrawService::new(disp.clone(), shaders),
				input_service: InputService::new(disp, ev_loop),
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
	events_loop: EventsLoop,
	events: Vec<Event>,
}

use glium::glutin::dpi::{LogicalSize, LogicalPosition};
impl InputService {
	pub fn new(display: Rc<Display>, events_loop: EventsLoop) -> InputService {
		let mut ret = InputService {
			display: display,
			events_loop: events_loop,
			events: Vec::new(),
		};
		ret.flush_event_queue();
		ret
	}

	pub fn flush_event_queue(&mut self) {
		let mut new_events = Vec::new();
		self.events_loop.poll_events(|ev| new_events.push(ev));
		replace(&mut self.events, new_events);
	}

	pub fn events(&self) -> Iter<Event> {
		self.events.iter()
	}

	pub fn size(&self) -> Option<LogicalSize> {
		self.display.gl_window().get_inner_size()
	}

	pub fn set_cursor_position(&self, pos: LogicalPosition) {
		self.display.gl_window().set_cursor_position(pos);
	}
}

pub struct DrawService {
	display: Rc<Display>,
	frame: Frame,
	program: Program,
	perspective: Perspective3<f32>,
}

impl Drop for DrawService {
	fn drop(&mut self) {
		self.frame.set_finish();
	}
}

impl DrawService {
	fn build_perspective(frame: &Frame) -> Perspective3<f32> {
		let (width, height) = frame.get_dimensions();

		let fov: f32 = ::std::f32::consts::PI / 3.0;
		let zfar = 1024.0;
		let znear = 0.001;

		Perspective3::new(width as f32 / height as f32, fov, znear, zfar)
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

const MOUSE_SENSITIVITY:  f32 = 0.00000001;
const MOTION_SENSITIVITY: f32 = 0.00001;
const MOTION_SENSITIVITY_FAST: f32 = 0.001;

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
			use glium::glutin::dpi::LogicalPosition;
			match ev {
				&Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					..
				} => return UpdateResult::Quit,   // the window has been closed by the user

				&Event::WindowEvent {
					event: WindowEvent::KeyboardInput {
						input: input,
						..
					},
					..
				} => {
					let ::glium::glutin::KeyboardInput {
						virtual_keycode: opt_key,
						state: state,
						..
					} = input;
					match opt_key {
						None => (),
						Some(key) => match key {
							VirtualKeyCode::Escape => return UpdateResult::Quit,
							code => match state {
								ElementState::Pressed => { self.keys_down.insert(code); },
								ElementState::Released => { self.keys_down.remove(&code); },
							},
						}
					}
				},

				&Event::WindowEvent {
					event: WindowEvent::CursorMoved{
						position: LogicalPosition{x: raw_x, y: raw_y},
						..
					},
					..
				} => {
					let size = services.input_service.size().unwrap();
					let mid: LogicalPosition = (size.width / 2.0, size.height / 2.0).into();
					services.input_service.set_cursor_position(mid);

					let (delta_x, delta_y) = (raw_x - mid.x, raw_y - mid.y);

					let dir = &mut self.camera.direction;
					let up  = &self.camera.up;

					*dir = Rotation3::new(up               * -delta_x as f32 * MOUSE_SENSITIVITY * time_elapsed.num_microseconds().unwrap() as f32)
					     * Rotation3::new(up.cross(dir) * -delta_y as f32 * MOUSE_SENSITIVITY * time_elapsed.num_microseconds().unwrap() as f32)
					     * (*dir);

					*dir = dir.normalize();

					dir[2] = f32::max(-0.9, f32::min(0.9, dir[2]));
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
				self.camera.position -= self.camera.direction.cross(&self.camera.up) * time_elapsed.num_microseconds().unwrap() as f32 * dolly_speed;
			},
			(false, true) => {
				self.camera.position -= -1.0 * self.camera.direction.cross(&self.camera.up) * time_elapsed.num_microseconds().unwrap() as f32 * dolly_speed;
			},
		}

		match (self.keys_down.contains(&VirtualKeyCode::W), self.keys_down.contains(&VirtualKeyCode::S)) {
			(true, true) => (),
			(false, false) => (),

			(true, false) => {
				self.camera.position -= -1.0 * self.camera.direction * time_elapsed.num_microseconds().unwrap() as f32 * dolly_speed;
			},
			(false, true) => {
				self.camera.position -= self.camera.direction * time_elapsed.num_microseconds().unwrap() as f32 * dolly_speed;
			},
		}

		match (self.keys_down.contains(&VirtualKeyCode::E), self.keys_down.contains(&VirtualKeyCode::Q)) {
			(true, true) => (),
			(false, false) => (),

			(true, false) => {
				self.camera.position -= -1.0 * self.camera.up * time_elapsed.num_microseconds().unwrap() as f32 * dolly_speed;

			},
			(false, true) => {
				self.camera.position -= self.camera.up * time_elapsed.num_microseconds().unwrap() as f32 * dolly_speed;
			},
		}

		UpdateResult::None
	}

	fn draw(&self, draw_service: &mut DrawService) {
		self.region.draw(&self.block_render_types, draw_service, self.camera.to_isometry().to_homogeneous());
	}
}
