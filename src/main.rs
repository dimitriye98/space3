#[macro_use]
extern crate glium;

#[macro_use]
extern crate bitflags;

extern crate time;
extern crate nalgebra as na;
extern crate rand;
extern crate noise;
extern crate ndarray;

mod gl_util;
mod block;
mod engine;

fn main() {
	use time::PreciseTime;

	use engine::{Game, StatePlaying, DrawService};

	let vertex_shader_src   = include_str!("standard.vert");
	let fragment_shader_src = include_str!("standard.frag");

	let mut events_loop = glium::glutin::EventsLoop::new();
	let window = glium::glutin::WindowBuilder::new();
	let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
	let display = glium::Display::new(window, context, &events_loop)
			.expect("Failed to initialize display");

	let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

	println!("Should live here");

	let mut game = Game::new(Box::new(StatePlaying::new()), display, events_loop, program);

	let mut last_tick: PreciseTime = PreciseTime::now();

	while game.is_running() {
		let old = last_tick;
		last_tick = PreciseTime::now();
		let time_elapsed = old.to(last_tick);

		game.update(&time_elapsed);
		game.draw();
	}
}
