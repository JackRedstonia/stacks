use crate::prelude::*;

pub struct Throbber {
    pub take_input: bool,
    pub paint: Paint,
    radius: LayoutDimension,
    size: Size,
    rad: scalar,
}

impl Throbber {
    pub fn new(radius: LayoutDimension, paint: Paint) -> Wrap<Self> {
        Self {
            take_input: false,
            radius,
            size: Size::new_empty(),
            paint,
            rad: 0.0,
        }
        .into()
    }
}

impl Widget for Throbber {
    fn load(&mut self, _state: &mut WidgetState, _stack: &mut ResourceStack) {}

    fn update(&mut self, _state: &mut WidgetState) {}

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        self.take_input
            && event.position().map_or(false, |p| {
                let s = self.size.width.min(self.size.height);
                Rect::from_wh(s, s).contains(p)
            })
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        (
            LayoutSize {
                width: self.radius,
                height: self.radius,
            },
            false,
        )
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        let stroke_width = self.paint.stroke_width();
        let s = self.size.width.min(self.size.height) - stroke_width;
        canvas.draw_arc(
            Rect::from_wh(s, s)
                .with_offset(Vector::new(stroke_width, stroke_width) * 0.5),
            self.rad,
            240.0,
            false,
            &self.paint,
        );
        self.rad += State::last_update_time_draw().as_secs_f32() * 720.0;
    }
}
