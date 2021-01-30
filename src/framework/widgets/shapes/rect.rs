use crate::prelude::*;

pub struct Rectangle {
    pub layout_size: LayoutSize,
    size: Size,
    pub paint: Paint,
}

impl Rectangle {
    pub fn new(size: LayoutSize, paint: Paint) -> Self {
        Self {
            layout_size: size,
            size: Size::new_empty(),
            paint,
        }
    }
}

impl Widget for Rectangle {
    fn input(&mut self, _wrap: &mut WidgetState, event: &InputEvent) -> bool {
        event
            .position()
            .map_or(false, |p| Rect::from_size(self.size).contains(p))
    }

    fn size(&mut self, _wrap: &mut WidgetState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _wrap: &mut WidgetState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WidgetState, canvas: &mut Canvas) {
        canvas.draw_rect(Rect::from_size(self.size), &self.paint);
    }
}
