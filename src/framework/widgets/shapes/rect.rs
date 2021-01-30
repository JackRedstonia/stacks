use crate::prelude::*;
use game::InputEvent;
use skia::{Canvas, Contains, Paint, Rect as SkRect, Size};

pub struct Rect {
    pub layout_size: LayoutSize,
    size: Size,
    pub paint: Paint,
}

impl Rect {
    pub fn new(size: LayoutSize, paint: Paint) -> Self {
        Self {
            layout_size: size,
            size: Size::new_empty(),
            paint,
        }
    }
}

impl Widget for Rect {
    fn input(&mut self, _wrap: &mut WidgetState, event: &InputEvent) -> bool {
        event
            .position()
            .map_or(false, |p| SkRect::from_size(self.size).contains(p))
    }

    fn size(&mut self, _wrap: &mut WidgetState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _wrap: &mut WidgetState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WidgetState, canvas: &mut Canvas) {
        canvas.draw_rect(SkRect::from_size(self.size), &self.paint);
    }
}
