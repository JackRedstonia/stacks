use super::super::{LayoutSize, Widget, WrapState};
use crate::game::InputEvent;
use crate::skia::{Canvas, Contains, Paint, Point, Rect as SkRect, Size};

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
                let p: Point = (p.x, p.y).into();
                SkRect::from_size(self.size).contains(p)
            })
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        self.layout_size
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        canvas.draw_rect(SkRect::from_size(self.size), &self.paint);
    }
}
