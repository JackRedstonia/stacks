use super::Component;
use crate::game::{
    input::{InputEvent, InputState},
    time::TimeState,
};
use crate::canvas::Canvas;
use skia_safe::{scalar, Matrix, Point};
use skulpin_renderer::skia_safe;
use skulpin_renderer_winit::winit::dpi::LogicalPosition;

pub struct Parallax<T: Component> {
    pub inner: T,
    pub last_mouse_position: Point,
    pub interpolated_mouse_position: Point,
}

impl<T: Component> Parallax<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            last_mouse_position: (0.0, 0.0).into(),
            interpolated_mouse_position: (0.0, 0.0).into(),
        }
    }

    fn calc_parallax(&self) -> Matrix {
        Matrix::translate(self.interpolated_mouse_position * 0.05)
    }

    fn interpolate_parallax(&mut self, factor: scalar) {
        let diff = self.last_mouse_position - self.interpolated_mouse_position;
        if diff.distance_to_origin() < 1.0 {
            self.interpolated_mouse_position = self.last_mouse_position;
        } else {
            self.interpolated_mouse_position += diff * factor;
        }
    }
}

impl<T: Component> Component for Parallax<T> {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState) {
        self.inner.update(input_state, time_state);
    }

    fn draw(&mut self, input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas) {
        self.interpolate_parallax(time_state.last_update_time().as_secs_f32() * 20.0);
        canvas.save();
        canvas.concat(self.calc_parallax());
        self.inner.draw(input_state, time_state, canvas);
        canvas.restore();
    }

    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: &InputEvent) {
        match event {
            InputEvent::MouseMove(LogicalPosition { x, y }) => {
                self.last_mouse_position = (*x, *y).into();
            }
            _ => {}
        }
        if let Some(event) = event.reverse_map_position(self.calc_parallax()) {
            self.inner.input(input_state, time_state, &event);
        }
    }
}
