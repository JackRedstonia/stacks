use super::{LayoutDimension, LayoutSize, Widget, WrapState};
use crate::game::State;
use crate::skia;
use skia::{scalar, Canvas, Color4f, Paint, PaintStyle, Rect, Size};

pub struct Metrics {
    pub radius: LayoutDimension,
    size: Size,
    update_paint: Paint,
    draw_paint: Paint,
    update_accm: scalar,
    update_count: scalar,
}

impl Metrics {
    pub fn new(radius: LayoutDimension) -> Self {
        Self {
            radius,
            size: Size::new_empty(),
            update_paint: {
                let mut p = Paint::new(Color4f::new(0.0, 1.0, 0.0, 1.0), None);
                p.set_stroke_width(8.0);
                p.set_anti_alias(true);
                p.set_style(PaintStyle::Stroke);
                p
            },
            draw_paint: {
                let mut p = Paint::new(Color4f::new(1.0, 0.0, 0.0, 1.0), None);
                p.set_stroke_width(8.0);
                p.set_anti_alias(true);
                p.set_style(PaintStyle::Stroke);
                p
            },
            update_accm: 0.0,
            update_count: 0.0,
        }
    }
}

impl Widget for Metrics {
    fn update(&mut self, _wrap: &mut WrapState) {
        self.update_accm += State::last_update_time().as_secs_f32();
        self.update_count += 1.0;
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        LayoutSize {
            width: self.radius,
            height: self.radius,
        }
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        let s = self.size.width.min(self.size.height);
        let oval = Rect::from_wh(s, s);
        let draw_time = State::last_update_time_draw().as_secs_f32();
        let update_time = self.update_accm / self.update_count;
        canvas.draw_arc(
            oval,
            0.0,
            360.0 * 100000.0 * update_time,
            false,
            &self.update_paint,
        );
        canvas.draw_arc(
            oval.with_offset((200.0, 0.0)),
            0.0,
            360.0 * 50.0 * draw_time,
            false,
            &self.draw_paint,
        );

        self.update_accm = 0.0;
    }
}
