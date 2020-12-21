mod builder;
mod canvas;
mod default_font_set;
mod input;
mod runner;
mod time;

pub use builder::Builder;
pub use canvas::{Canvas, FontSet};
pub use input::{EventHandleResult, InputEvent, InputState};
pub use runner::{Error, State};
pub use time::TimeState;

pub trait Game {
    fn update(&mut self);
    fn draw(&mut self, canvas: &mut canvas::Canvas);
    fn input(&mut self, event: InputEvent);
    fn close(&mut self);
    fn crash(&mut self, err: runner::Error);
}
