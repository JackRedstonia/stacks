mod composite;
mod metrics;
mod parallax;
pub mod shapes;
mod text;
mod transform;

pub use composite::Composite;
pub use metrics::Metrics;
pub use parallax::Parallax;
pub use text::{Font, FontStyle, Text};
pub use transform::Transform;

use crate::game::{Canvas, InputEvent, InputState, TimeState};

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
