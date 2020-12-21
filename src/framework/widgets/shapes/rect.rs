use super::super::{LayoutSize, Widget};
use crate::game::{Canvas, InputEvent};
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

impl Widget for Rect {
    fn input(&mut self, event: &InputEvent, size: Size) -> bool {
        self.take_input
            && event.position().map_or(false, |p| {
                let p: Point = (p.x, p.y).into();
                skia::Rect::from_size(size).contains(p)
            })
    }

    fn size(&mut self) -> LayoutSize {
        self.size
    }

    fn draw(&mut self, canvas: &mut Canvas, size: Size) {
        canvas.draw_rect(skia::Rect::from_size(size), &self.paint);
    }
}
