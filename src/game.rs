mod builder;
mod input;
mod runner;
mod time;

pub use builder::Builder;
pub use input::{EventHandleResult, InputEvent, InputState, ScrollAmount};
pub use runner::{GameError, RunnerError, State, ID};
pub use time::TimeState;

pub trait Game {
    fn update(&mut self);
    fn draw(&mut self, canvas: &mut crate::skia::Canvas);
    fn set_size(&mut self, size: crate::skia::Size);
    fn input(&mut self, event: InputEvent);
    fn close(&mut self);
    fn crash(&mut self, err: runner::GameError);
}
