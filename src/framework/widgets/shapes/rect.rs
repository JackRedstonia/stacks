use crate::prelude::*;
use game::InputEvent;
use skia::{Canvas, Contains, Paint, Rect as SkRect, Size};

pub struct Rect {
    pub layout_size: LayoutSize,
    size: Size,
    pub paint: Paint,
    pub take_input: bool,
}

impl Rect {
    pub fn new(size: LayoutSize, paint: Paint, take_input: bool) -> Self {
        Self {
            layout_size: size,
            size: Size::new_empty(),
            paint,
            take_input,
        }
    }
}

impl Widget for Rect {
    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent) -> bool {
        self.take_input
            && event.position().map_or(false, |p| {
                SkRect::from_size(self.size).contains(p.to_point())
            })
    }

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        canvas.draw_rect(SkRect::from_size(self.size), &self.paint);
    }
}
