pub mod shapes;

use super::application::{
    input::{InputEvent, InputState},
    time::TimeState,
};
use crate::canvas::Canvas;

pub trait Component {
    fn update(&mut self, _input_state: &InputState, _time_state: &TimeState);
    fn draw(&mut self, _input_state: &InputState, _time_state: &TimeState, _canvas: &mut Canvas);
    fn input(&mut self, _input_state: &InputState, _time_state: &TimeState, _event: &InputEvent);
}
