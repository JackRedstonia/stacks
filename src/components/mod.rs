mod composite;
pub mod shapes;
mod transform;
mod text;

pub use composite::Composite;
pub use transform::Transform;
pub use text::{Font, FontStyle, Text};

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
