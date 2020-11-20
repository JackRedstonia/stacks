use super::super::Component;
use crate::game::{Canvas, InputEvent, InputState, TimeState};
use skia_safe::{scalar, Paint, Rect};
use skulpin_renderer::skia_safe;

pub struct Throbber {
    pub radius: scalar,
    pub paint: Paint,
    rad: scalar,
}

impl Throbber {
    pub fn new(radius: scalar, paint: Paint) -> Self {
        Self {
            radius,
            paint,
            rad: 0.0,
        }
    }
}

impl Component for Throbber {
    fn update(&mut self, _input_state: &InputState, _time_state: &TimeState) {}

    fn draw(&mut self, _input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas) {
        canvas.draw_arc(
            Rect {
                left: -self.radius,
                top: -self.radius,
                right: self.radius,
                bottom: self.radius,
            },
            self.rad,
            240.0,
            false,
            &self.paint,
        );
        self.rad += time_state.last_update_time().as_secs_f32() * 720.0;
    }

    fn input(&mut self, _input_state: &InputState, _time_state: &TimeState, _event: &InputEvent) {}
}
