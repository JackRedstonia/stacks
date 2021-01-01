use crate::prelude::*;
use game::{InputEvent, State};
use skia::{scalar, Canvas, Contains, Paint, Rect, Size, Vector};

pub struct Throbber {
    pub radius: LayoutDimension,
    size: Size,
    pub paint: Paint,
    pub take_input: bool,
    rad: scalar,
}

impl Throbber {
    pub fn new(radius: LayoutDimension, paint: Paint, take_input: bool) -> Self {
        Self {
            radius,
            size: Size::new_empty(),
            paint,
            take_input,
            rad: 0.0,
        }
    }
}

impl Widget for Throbber {
    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent) -> bool {
        self.take_input
            && event.position().map_or(false, |p| {
                let s = self.size.width.min(self.size.height);
                Rect::from_wh(s, s).contains(p.to_point())
            })
    }

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        (
            LayoutSize {
                width: self.radius,
                height: self.radius,
            },
            false,
        )
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        let stroke_width = self.paint.stroke_width();
        let s = self.size.width.min(self.size.height) - stroke_width;
        canvas.draw_arc(
            Rect::from_wh(s, s).with_offset(Vector::new(stroke_width, stroke_width) * 0.5),
            self.rad,
            240.0,
            false,
            &self.paint,
        );
        self.rad += State::last_update_time_draw().as_secs_f32() * 720.0;
    }
}
