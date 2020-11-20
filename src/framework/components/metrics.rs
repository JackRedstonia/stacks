use super::Component;
use crate::game::{Canvas, InputEvent, InputState, TimeState};
use skia_safe::{scalar, Paint, Rect};
use skulpin_renderer::skia_safe;

pub struct Metrics {
    pub radius: scalar,
    update_paint: Paint,
    draw_paint: Paint,
    update_accm: scalar,
    update_count: scalar,
}

impl Metrics {
    pub fn new(radius: scalar) -> Self {
        Self {
            radius,
            update_paint: {
                let mut p =
                    skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
                p.set_stroke_width(8.0);
                p.set_anti_alias(true);
                p.set_style(skia_safe::PaintStyle::Stroke);
                p
            },
            draw_paint: {
                let mut p =
                    skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 0.0, 0.0, 1.0), None);
                p.set_stroke_width(8.0);
                p.set_anti_alias(true);
                p.set_style(skia_safe::PaintStyle::Stroke);
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

    fn draw(&mut self, _input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas) {
        let oval = Rect {
            left: -self.radius,
            top: -self.radius,
            right: self.radius,
            bottom: self.radius,
        };
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

    fn input(&mut self, _input_state: &InputState, _time_state: &TimeState, _event: &InputEvent) {}
}
