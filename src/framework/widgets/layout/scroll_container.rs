use crate::prelude::*;

pub struct ScrollContainer {
    offset: scalar,
    target_offset: scalar,
    size: Size,
    layout_size: LayoutSize,
    child_size: Size,
    child_layout_size: LayoutSize,
    matrix: Matrix,
}

impl ScrollContainer {
    pub fn new(size: LayoutSize) -> Wrap<Self> {
        Self {
            offset: 0.0,
            target_offset: 0.0,
            size: Size::default(),
            layout_size: size,
            child_size: Size::default(),
            child_layout_size: LayoutSize::ZERO,
            matrix: Matrix::default(),
        }
        .into()
    }

    fn scroll(&mut self, i: i32) {
        let offset = i as scalar * 50.0;
        self.target_offset += offset;
        self.rescroll();
    }

    fn rescroll(&mut self) {
        let max = (self.child_size.height - self.size.height).max(0.0);
        self.target_offset = (self.target_offset).min(0.0).max(-max);
    }

    fn interpolate_scroll(&mut self) {
        let delta = self.target_offset - self.offset;
        if delta.abs() < 0.1 {
            self.offset = self.target_offset;
        } else {
            let t = State::last_update_time_draw().as_secs_f32();
            self.offset += t * delta * 30.0;
        }
        self.matrix = Matrix::translate((0.0, self.offset));
    }
}

impl Widget for ScrollContainer {
    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        if let Some(child) = state.child() {
            if let Some(p) = event.position() {
                if event.consumable() && !Rect::from_size(self.size).contains(p)
                {
                    return false;
                }
            }

            let taken = event
                .reverse_map_position(self.matrix)
                .map(|e| child.input(&e) && e.consumable())
                .unwrap_or(false);
            if !taken {
                if let InputEvent::MouseScroll(i, _) = event {
                    self.scroll(*i);
                    return true;
                }
            }
            return taken;
        }
        false
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        let changed = state
            .child()
            .map(|e| {
                let s = e.size();
                let changed = self.child_layout_size == s.0;
                self.child_layout_size = s.0;
                changed || s.1
            })
            .unwrap_or(false);
        (self.layout_size, changed)
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.size = size;
        if let Some(child) = state.child() {
            self.child_size = self.child_layout_size.layout_one(size);
            child.set_size(self.child_size);
            self.rescroll();
        }
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut skia::Canvas) {
        self.interpolate_scroll();
        if let Some(child) = state.child() {
            canvas.save();
            canvas.clip_rect(Rect::from_size(self.size), None, true);
            canvas.concat(&self.matrix);
            child.draw(canvas);
            canvas.restore();
        }
    }
}
