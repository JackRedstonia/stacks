mod builder;
mod canvas;
mod default_font_set;
mod input;
mod runner;
mod time;

pub use builder::Builder;
pub use canvas::FontSet;
pub use input::{EventHandleResult, InputEvent, InputState};
pub use runner::{Error, State, ID};
pub use time::TimeState;

pub trait Game: 'static + Send {
    fn update(&mut self);
    fn draw(&mut self, canvas: &mut crate::skia::Canvas);
    fn set_size(&mut self, size: crate::skia::Size);
    fn input(&mut self, event: InputEvent);
    fn close(&mut self);
    fn crash(&mut self, err: runner::Error);
}
