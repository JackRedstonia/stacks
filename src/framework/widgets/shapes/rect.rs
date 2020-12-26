use super::super::{LayoutSize, Widget, WrapState};
use crate::game::InputEvent;
use crate::skia::{Contains, Paint, Point, Size, Canvas, Rect as SkRect};

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

impl Widget for Rect {
    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent, size: Size) -> bool {
        self.take_input
            && event.position().map_or(false, |p| {
                let p: Point = (p.x, p.y).into();
                SkRect::from_size(size).contains(p)
            })
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        self.size
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas, size: Size) {
        canvas.draw_rect(SkRect::from_size(size), &self.paint);
    }
}
