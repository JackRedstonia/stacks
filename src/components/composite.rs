use super::Component;
use crate::game::{Canvas, InputEvent, InputState, TimeState};

pub struct Composite<T: Component> {
    pub inner: Vec<T>,
}

impl<T: Component> Component for Composite<T> {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState) {
        for i in &mut self.inner {
            i.update(input_state, time_state);
        }
    }

    fn draw(&mut self, input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas) {
        for i in &mut self.inner {
            i.draw(input_state, time_state, canvas);
        }
    }

    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: &InputEvent) {
        for i in self.inner.iter_mut().rev() {
            i.input(input_state, time_state, &event);
        }
    }
}
