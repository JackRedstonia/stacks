mod builder;
mod canvas;
mod default_font_set;
mod input;
mod runner;
mod time;

pub use builder::GameBuilder;
pub use canvas::{Canvas, FontSet};
pub use input::{EventHandleResult, InputEvent, InputState};
pub use runner::GameError;
pub use time::TimeState;

pub trait Game {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState);
    fn draw(
        &mut self,
        input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut canvas::Canvas,
    );
    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: InputEvent);
    fn close(&mut self);
    fn crash(&mut self, err: runner::GameError);
}
