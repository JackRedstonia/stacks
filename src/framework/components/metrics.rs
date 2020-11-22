use super::{Component, LayoutDimension, LayoutSize};
use crate::game::{Canvas, InputEvent, InputState, TimeState};
use crate::skia;
use skia::{scalar, Paint, Size};

pub struct Metrics {
    pub radius: LayoutDimension,
    update_paint: Paint,
    draw_paint: Paint,
    update_accm: scalar,
    update_count: scalar,
}

impl Metrics {
    pub fn new(radius: LayoutDimension) -> Self {
        Self {
            radius,
            update_paint: {
                let mut p = skia::Paint::new(skia::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
                p.set_stroke_width(8.0);
                p.set_anti_alias(true);
                p.set_style(skia::PaintStyle::Stroke);
                p
            },
            draw_paint: {
                let mut p = skia::Paint::new(skia::Color4f::new(1.0, 0.0, 0.0, 1.0), None);
                p.set_stroke_width(8.0);
                p.set_anti_alias(true);
                p.set_style(skia::PaintStyle::Stroke);
                p
            },
            update_accm: 0.0,
            update_count: 0.0,
        }
    }
}

impl Component for Metrics {
    fn update(&mut self, _input_state: &InputState, time_state: &TimeState) {
        self.update_accm += time_state.last_update_time().as_secs_f32();
        self.update_count += 1.0;
    }

    fn input(
        &mut self,
        _input_state: &InputState,
        _time_state: &TimeState,
        _event: &InputEvent,
        _size: Size,
    ) -> bool {
        false
    }

    fn size(&mut self, _input_state: &InputState, _time_state: &TimeState) -> LayoutSize {
        LayoutSize {
            width: self.radius,
            height: self.radius,
        }
    }

    fn draw(
        &mut self,
        _input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut Canvas,
        size: Size,
    ) {
        let s = size.width.min(size.height);
        let oval = skia::Rect::from_wh(s, s);
        let draw_time = time_state.last_update_time().as_secs_f32();
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
