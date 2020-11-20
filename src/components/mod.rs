mod composite;
pub mod shapes;
mod transform;
mod text;
mod parallax;

pub use composite::Composite;
pub use transform::Transform;
pub use text::{Font, FontStyle, Text};
pub use parallax::Parallax;

use super::application::{
    input::{InputEvent, InputState},
    time::TimeState,
};
use crate::canvas::Canvas;

pub trait Component {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState);
    fn draw(&mut self, input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas);
    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: &InputEvent);
}

impl Component for Box<dyn Component + Send> {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState) {
        self.as_mut().update(input_state, time_state);
    }

    fn draw(&mut self, input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas) {
        self.as_mut().draw(input_state, time_state, canvas);
    }

    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: &InputEvent) {
        self.as_mut().input(input_state, time_state, event);
    }
}
