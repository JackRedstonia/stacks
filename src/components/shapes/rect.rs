use super::super::Component;
use crate::application::{
    input::{InputEvent, InputState},
    time::TimeState,
};
use crate::canvas::Canvas;
use skia_safe::{Paint, Size};
use skulpin_renderer::skia_safe;

pub struct Rect {
    pub rect: skia_safe::Rect,
    pub paint: Paint,
}

impl Rect {
    pub fn new(size: impl Into<Size>, paint: Paint) -> Self {
        let Size { width, height  } = size.into();
        Self {
            rect: skia_safe::Rect {
                left: 0.0,
                top: 0.0,
                right: width,
                bottom: height,
            },
            paint,
        }
    }
}

impl Component for Rect {
    fn update(&mut self, _input_state: &InputState, _time_state: &TimeState) {}

    fn draw(&mut self, _input_state: &InputState, _time_state: &TimeState, canvas: &mut Canvas) {
        canvas.draw_rect(self.rect, &self.paint);
    }

    fn input(&mut self, _input_state: &InputState, _time_state: &TimeState, _event: &InputEvent) {}
}
