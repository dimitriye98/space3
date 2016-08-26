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
	use glium::DisplayBuild;
	use time::PreciseTime;

	use engine::{Game, StatePlaying, DrawService};

	let vertex_shader_src   = include_str!("standard.vert");
	let fragment_shader_src = include_str!("standard.frag");

	let display = glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap();

	let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

	let mut game = Game::new(Box::new(StatePlaying::new()), display, program);

	let mut last_tick: PreciseTime = PreciseTime::now();

	while game.is_running() {
		let old = last_tick;
		last_tick = PreciseTime::now();
		let time_elapsed = old.to(last_tick);

		game.update(&time_elapsed);
		game.draw();
	}
}
