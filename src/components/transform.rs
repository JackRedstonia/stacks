use super::Component;
use crate::game::{Canvas, InputEvent, InputState, TimeState};
use skia_safe::Matrix;
use skulpin_renderer::skia_safe;

pub struct Transform<T: Component> {
    pub inner: T,
    pub matrix: Matrix,
}

impl<T: Component> Component for Transform<T> {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState) {
        self.inner.update(input_state, time_state);
    }

    fn draw(&mut self, input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas) {
        canvas.save();
        canvas.concat(self.matrix);
        self.inner.draw(input_state, time_state, canvas);
        canvas.restore();
    }

    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: &InputEvent) {
        if let Some(event) = event.reverse_map_position(self.matrix) {
            self.inner.input(input_state, time_state, &event);
        }
    }
}
