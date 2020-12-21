use super::super::{LayoutDimension, LayoutSize, Widget};
use crate::game::{Canvas, InputEvent, State};
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

impl Widget for Throbber {
    fn input(&mut self, event: &InputEvent, size: Size) -> bool {
        self.take_input
            && event.position().map_or(false, |p| {
                let p: Point = (p.x, p.y).into();
                let s = size.width.min(size.height);
                skia::Rect::from_wh(s, s).contains(p)
            })
    }

    fn size(&mut self) -> LayoutSize {
        LayoutSize {
            width: self.radius,
            height: self.radius,
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, size: Size) {
        let stroke_width = self.paint.stroke_width();
        let s = size.width.min(size.height) - stroke_width;
        canvas.draw_arc(
            skia::Rect::from_wh(s, s)
                .with_offset(skia::Vector::new(stroke_width, stroke_width) * 0.5),
            self.rad,
            240.0,
            false,
            &self.paint,
        );
        self.rad += State::last_update_time_draw().as_secs_f32() * 720.0;
    }
}
