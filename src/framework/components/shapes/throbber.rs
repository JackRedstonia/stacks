use super::super::{Component, LayoutDimension, LayoutSize};
use crate::game::{Canvas, InputEvent, InputState, TimeState};
use crate::skia;
use skia::{scalar, Contains, Paint, Point, Size};

pub struct Throbber {
    pub radius: LayoutDimension,
    pub paint: Paint,
    pub take_input: bool,
    rad: scalar,
}

impl Throbber {
    pub fn new(radius: LayoutDimension, paint: Paint, take_input: bool) -> Self {
        Self {
            radius,
            paint,
            take_input,
            rad: 0.0,
        }
    }
}

impl Component for Throbber {
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
                let s = size.width.min(size.height);
                skia::Rect::from_wh(s, s).contains(p)
            })
    }

    fn size(&mut self, _input_state: &InputState, _time_state: &TimeState) -> LayoutSize {
        LayoutSize {
            width: self.radius,
            height: self.radius,
        }
    }

    fn draw(
        &mut self,
        _input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut Canvas,
        size: Size,
    ) {
        let s = size.width.min(size.height);
        canvas.draw_arc(
            skia::Rect::from_wh(s, s),
            self.rad,
            240.0,
            false,
            &self.paint,
        );
        self.rad += time_state.last_update_time().as_secs_f32() * 720.0;
    }
}
