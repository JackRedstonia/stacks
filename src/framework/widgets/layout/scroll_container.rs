use crate::prelude::*;

pub struct ScrollContainer<T: Widget + ?Sized> {
    child: Wrap<T>,
    offset: scalar,
    target_offset: scalar,
    size: Size,
    layout_size: LayoutSize,
    child_size: Size,
    child_layout_size: LayoutSize,
    matrix: Matrix,
}

impl<T: Widget + ?Sized> ScrollContainer<T> {
    pub fn new(child: Wrap<T>, size: LayoutSize) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            child,
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
        self.target_offset = self.target_offset.clamp(-max, 0.0);
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

impl<T: Widget + ?Sized> Widget for ScrollContainer<T> {
    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack) {
        self.child.load(stack);
    }

    fn update(&mut self, state: &mut WidgetState) {
        self.child.update();
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        if let Some(p) = event.position() {
            if event.is_consumable() && !Rect::from_size(self.size).contains(p)
            {
                return false;
            }
        }

        let taken = event
            .reverse_map_position(self.matrix)
            .map(|e| self.child.input(&e) && e.is_consumable())
            .unwrap_or(false);
        if !taken {
            if let InputEvent::MouseScroll(i, _) = event {
                self.scroll(*i);
                return true;
            }
        }
        taken
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        let s = self.child.size();
        let changed = self.child_layout_size == s.0;
        self.child_layout_size = s.0;
        (self.layout_size, changed || s.1)
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.size = size;
        self.child_size = self.child_layout_size.layout_one(size);
        self.child.set_size(self.child_size);
        self.rescroll();
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut skia::Canvas) {
        self.interpolate_scroll();
        canvas.save();
        canvas.clip_rect(Rect::from_size(self.size), None, true);
        canvas.concat(&self.matrix);
        self.child.draw(canvas);
        canvas.restore();
    }
}
