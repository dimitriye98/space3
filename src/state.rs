use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::Window;

use time::Duration;

pub enum UpdateResult {
	None,
	Quit,
	ChangeState(Box<GameState>),
}

pub enum Event {
	GlutinEvent(::glium::glutin::Event),
}

pub trait GameState {
	fn entered(&mut self) -> ();
	fn leaving(&mut self) -> ();

	fn update(&mut self, events: &mut Iterator<Item=Event>, time_elapsed: &Duration, window: &Window) -> UpdateResult;
	fn draw(&self, facade: &GlutinFacade) -> ();
}

pub trait StateManager {
	fn swap_state(&mut self, state: Box<GameState>) -> Box<GameState>;

	fn update(&mut self, events: &mut Iterator<Item=Event>, time_elapsed: &Duration, window: &Window) -> ();
	fn draw(&self, facade: &GlutinFacade) -> ();
}

