#[macro_use]
extern crate glium;

#[macro_use]
extern crate bitflags;

extern crate time;
extern crate nalgebra as na;

mod gl_util;
mod block;
mod state;
mod engine;

fn main() {
	use glium::{DisplayBuild, Surface};
	use glium::glutin::{CursorState, MouseCursor};
	use time::{Duration, PreciseTime};

	use state::{StateManager, Event};
	use engine::{Game, StatePlaying};

	let vertex_shader_src   = include_str!("vertex.glsl");
	let fragment_shader_src = include_str!("fragment.glsl");

	let display = glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap();

	let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

	let mut game = Game::new(Box::new(StatePlaying::new(&display, program)));

	let mut last_tick: PreciseTime = PreciseTime::now();

	while game.is_running() {
		let old = last_tick;
		last_tick = PreciseTime::now();
		let time_elapsed = old.to(last_tick);

		game.update(&mut display.poll_events().map(|ev| Event::GlutinEvent(ev)), &time_elapsed, &display.get_window().unwrap());
		game.draw(&display);
	}
}
