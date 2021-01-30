use crate::prelude::*;
use game::InputEvent;
use skia::{Matrix, Size};

pub struct SizeFillContainer<T: Widget> {
    size: Size,
    child_layout_size: LayoutSize,
    matrix: Matrix,
    inner: Wrap<T>,
}

impl<T: Widget> SizeFillContainer<T> {
    pub fn new(inner: impl Into<Wrap<T>>) -> Self {
        FrameworkState::request_load();
        Self {
            inner: inner.into(),
            child_layout_size: LayoutSize::ZERO,
            matrix: Matrix::default(),
            size: Size::default(),
        }
    }
}

impl<T: Widget> Widget for SizeFillContainer<T> {
    fn load(&mut self, _wrap: &mut WidgetState, stack: &mut ResourceStack) {
        self.inner.load(stack);
    }

    fn update(&mut self, _wrap: &mut WidgetState) {
        self.inner.update();
    }

    fn input(&mut self, _wrap: &mut WidgetState, event: &InputEvent) -> bool {
        event
            .reverse_map_position(self.matrix)
            .map(|e| self.inner.input(&e))
            .unwrap_or(false)
    }

    fn size(&mut self, _wrap: &mut WidgetState) -> (LayoutSize, bool) {
        let (child_size, changed) = self.inner.size();
        self.child_layout_size = child_size;
        (LayoutSize::ZERO.expand_width().expand_height(), changed)
    }

    fn set_size(&mut self, _wrap: &mut WidgetState, size: Size) {
        self.size = size;
        let child_min = self.child_layout_size.get_min();
        let scale = (self.size.width / child_min.width).min(self.size.height / child_min.height);
        self.matrix = Matrix::scale((scale, scale));
        let child_max_size = size / scale;
        self.inner
            .set_size(self.child_layout_size.layout_one(child_max_size));
    }

    fn draw(&mut self, _wrap: &mut WidgetState, canvas: &mut skia::Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.inner.draw(canvas);
        canvas.restore();
    }
}
