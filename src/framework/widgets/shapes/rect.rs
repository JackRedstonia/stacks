use crate::prelude::*;

pub struct Rectangle {
    pub take_input: bool,
    pub paint: Paint,
    layout_size: LayoutSize,
    size: Size,
}

impl Rectangle {
    pub fn new(size: LayoutSize, paint: Paint) -> Wrap<Self> {
        Self {
            take_input: false,
            layout_size: size,
            size: Size::new_empty(),
            paint,
        }
        .into()
    }
}

impl Widget for Rectangle {
    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        self.take_input
            && event
                .position()
                .map_or(false, |p| Rect::from_size(self.size).contains(p))
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        canvas.draw_rect(Rect::from_size(self.size), &self.paint);
    }
}
