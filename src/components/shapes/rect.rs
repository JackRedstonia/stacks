use super::super::Component;
use crate::application::{
    input::{InputEvent, InputState},
    time::TimeState,
};
use crate::canvas::Canvas;
use skia_safe::Paint;
use skulpin_renderer::skia_safe;

pub struct Rect {
    pub rect: skia_safe::Rect,
    pub paint: Paint,
}

impl Component for Rect {
    fn update(&mut self, _input_state: &InputState, _time_state: &TimeState) {}

    fn draw(&mut self, _input_state: &InputState, _time_state: &TimeState, canvas: &mut Canvas) {
        canvas.draw_rect(self.rect, &self.paint);
    }

    fn input(&mut self, _input_state: &InputState, _time_state: &TimeState, _event: &InputEvent) {}
}
