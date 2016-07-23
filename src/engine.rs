use glium::backend::glutin_backend::GlutinFacade;
use glium::{Program, Surface};
use glium::glutin::Window;
use time::Duration;
use na::{Point3, Vector3, PerspectiveMatrix3, Rotation3, Norm, ToHomogeneous, Cross};


use state::{GameState, StateManager, UpdateResult, Event};
use gl_util::{Camera, SimpleCamera};
use block::{SimpleBlock, Chunk, CHUNK_SIZE};

pub struct Game {
	state: Box<GameState>,
	running: bool,
}

impl Game {
	pub fn new(start_state: Box<GameState>) -> Game {
		Game { state: start_state, running: true }
	}
	pub fn is_running(&self) -> bool { self.running }
	pub fn quit(&mut self) -> () { self.running = false }
}

impl StateManager for Game {
	fn swap_state(&mut self, state: Box<GameState>) -> Box<GameState> {
		self.state.leaving();
		let old_state = ::std::mem::replace(&mut self.state, state);
		self.state.entered();
		old_state
	}

	fn update(&mut self, events: &mut Iterator<Item=Event>, time_elapsed: &Duration, window: &Window) -> () {
		let result = self.state.update(events, time_elapsed, window);
		match result {
			UpdateResult::ChangeState(new_state) => { self.swap_state(new_state); },
			UpdateResult::Quit => self.quit(),
			UpdateResult::None => (),
		};
	}

	fn draw(&self, facade: &GlutinFacade) -> () {
		self.state.draw(facade);
	}
}

pub struct StatePlaying<'world> {
	chunk: Chunk<'world>,
	camera: SimpleCamera<f32>,
	program: Program,
	left: bool,
	right: bool,
	forward: bool,
	back: bool,
	up: bool,
	down: bool,
}

const MOUSE_SENSITIVITY:  f32 = 0.001;
const MOTION_SENSITIVITY: f32 = 0.001;
static BLOCK: SimpleBlock = SimpleBlock { color: [0.0, 0.8, 0.0] };

impl <'world> StatePlaying<'world> {
	pub fn new(display: &GlutinFacade, program: Program) -> StatePlaying<'world> {
		StatePlaying {
			chunk: Chunk { blocks: [[[&BLOCK; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] },
			camera: SimpleCamera {
				position:   Point3::new( 2.0, -1.0,  1.0),
				direction: Vector3::new(-2.0,  1.0,  1.0).normalize(),
				up:        Vector3::new( 0.0,  1.0,  0.0),
			},
			program: program,
			left: false,
			right: false,
			forward: false,
			back: false,
			up: false,
			down: false,
		}
	}
}

impl <'world> GameState for StatePlaying<'world> {
	fn entered(&mut self) -> () {}
	fn leaving(&mut self) -> () {}

	fn update(&mut self, events: &mut Iterator<Item=Event>, time_elapsed: &Duration, window: &Window) -> UpdateResult {
		for ev in events {
			use glium::glutin::Event as GlEvent;
			use glium::glutin::VirtualKeyCode;
			use glium::glutin::ElementState;
			match ev {
				Event::GlutinEvent(ev) => match ev {
					GlEvent::Closed => return UpdateResult::Quit,   // the window has been closed by the user
					GlEvent::KeyboardInput(pressed, _, key) => match key {
						None => (),
						Some(key) => match key {
							VirtualKeyCode::Escape => return UpdateResult::Quit,
							VirtualKeyCode::A => self.left = match pressed {
								ElementState::Pressed => true,
								ElementState::Released => false,
							},
							VirtualKeyCode::D => self.right = match pressed {
								ElementState::Pressed => true,
								ElementState::Released => false,
							},
							VirtualKeyCode::S => self.back = match pressed {
								ElementState::Pressed => true,
								ElementState::Released => false,
							},
							VirtualKeyCode::W => self.forward = match pressed {
								ElementState::Pressed => true,
								ElementState::Released => false,
							},
							VirtualKeyCode::LShift => self.down = match pressed {
								ElementState::Pressed => true,
								ElementState::Released => false,
							},
							VirtualKeyCode::Space => self.up = match pressed {
								ElementState::Pressed => true,
								ElementState::Released => false,
							},
	
							_ => ()
						}
					},
					GlEvent::MouseMoved((raw_x, raw_y)) => {
						let (size_x, size_y) = window.get_inner_size_points().unwrap();
						window.set_cursor_position((size_x / 2) as i32, (size_y / 2) as i32);
						let (delta_x, delta_y) = (raw_x - size_x as i32, raw_y - size_y as i32);
						self.camera.direction *= Rotation3::new(self.camera.up * (delta_x as f32) * MOUSE_SENSITIVITY);
						self.camera.direction *= Rotation3::new(self.camera.up.cross(&self.camera.direction) * (delta_y as f32) * MOUSE_SENSITIVITY);
					},

					_ => ()
				},
			}
		}

		match (self.left, self.right) {
			(true, true) => (),
			(false, false) => (),

			(true, false) => {
				self.camera.position -= self.camera.direction.cross(&self.camera.up) * time_elapsed.num_milliseconds() as f32 * MOTION_SENSITIVITY;
			},
			(false, true) => {
				self.camera.position -= -1.0 * self.camera.direction.cross(&self.camera.up) * time_elapsed.num_milliseconds() as f32 * MOTION_SENSITIVITY;
			},
		}

		match (self.forward, self.back) {
			(true, true) => (),
			(false, false) => (),

			(true, false) => {
				self.camera.position -= -1.0 * self.camera.direction * time_elapsed.num_milliseconds() as f32 * MOTION_SENSITIVITY;
			},
			(false, true) => {
				self.camera.position -= self.camera.direction * time_elapsed.num_milliseconds() as f32 * MOTION_SENSITIVITY;
			},
		}

		match (self.up, self.down) {
			(true, true) => (),
			(false, false) => (),

			(true, false) => {
				self.camera.position -= -1.0 * self.camera.up * time_elapsed.num_milliseconds() as f32 * MOTION_SENSITIVITY;

			},
			(false, true) => {
				self.camera.position -= self.camera.up * time_elapsed.num_milliseconds() as f32 * MOTION_SENSITIVITY;
			},
		}

		UpdateResult::None
	}

	fn draw(&self, display: &GlutinFacade) -> () {
		let mut target = display.draw();
		target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

		let uniforms = uniform! {
			model: [
				[0.01, 0.0, 0.0, 0.0],
				[0.0, 0.01, 0.0, 0.0],
				[0.0, 0.0, 0.01, 0.0],
				[0.0, 0.0, 1.0, 1.0f32],
			],
			view: self.camera.to_isometry().to_homogeneous().as_ref().clone(),
			perspective: {
				let (width, height) = target.get_dimensions();

				let fov: f32 = ::std::f32::consts::PI / 3.0;
				let zfar = 1024.0;
				let znear = 0.1;

				// perspective_matrix(width, height, fov, zfar, znear)
				PerspectiveMatrix3::new(width as f32 / height as f32, fov, znear, zfar).to_matrix().as_ref().clone()
			},
			u_light: [-1.0, 0.4, 0.9f32],
		};

		use glium::{DrawParameters, Depth};
		use glium::draw_parameters::{DepthTest, BackfaceCullingMode};
		use glium::index::{PrimitiveType, NoIndices};
		let params = DrawParameters {
			depth: Depth {
				test: DepthTest::IfLess,
				write: true,
				.. Default::default()
			},
			backface_culling: BackfaceCullingMode::CullClockwise,
			.. Default::default()
		};

		let vertices = self.chunk.build_mesh(display).unwrap();

		let indices = NoIndices(PrimitiveType::TrianglesList);

		target.draw(&vertices, &indices, &self.program, &uniforms, &params).unwrap();

		target.finish().unwrap();
	}
}

