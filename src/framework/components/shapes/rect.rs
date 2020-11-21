use super::super::{Component, LayoutSize};
use crate::game::{Canvas, InputEvent, InputState, TimeState};
use crate::skia;
use skia::{Contains, Paint, Point, Size};

pub struct Rect {
    pub size: LayoutSize,
    pub paint: Paint,
    pub take_input: bool,
}

impl Rect {
    pub fn new(size: LayoutSize, paint: Paint, take_input: bool) -> Self {
        Self {
            size,
            paint,
            take_input,
        }
    }
}

impl Component for Rect {
    fn update(&mut self, _input_state: &InputState, _time_state: &TimeState) {}

    fn input(
        &mut self,
        _input_state: &InputState,
        _time_state: &TimeState,
        event: &InputEvent,
        size: Size,
    ) -> bool {
        self.take_input
            && event.position().map_or(false, |p| {
                let p: Point = (p.x, p.y).into();
                skia::Rect::from_size(size).contains(p)
            })
    }

    fn size(&mut self, _input_state: &InputState, _time_state: &TimeState) -> LayoutSize {
        self.size
    }

    fn draw(
        &mut self,
        _input_state: &InputState,
        _time_state: &TimeState,
        canvas: &mut Canvas,
        size: Size,
    ) {
        canvas.draw_rect(skia::Rect::from_size(size), &self.paint);
    }
}
