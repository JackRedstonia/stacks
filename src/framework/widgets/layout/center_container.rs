use crate::prelude::*;
use game::InputEvent;
use skia::{Matrix, Size};

pub struct CenterContainer<T: Widget> {
    size: Size,
    child_layout_size: LayoutSize,
    matrix: Matrix,
    inner: Wrap<T>,
}

impl<T: Widget> CenterContainer<T> {
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

impl<T: Widget> Widget for CenterContainer<T> {
    fn load(&mut self, _wrap: &mut WrapState, stack: &mut ResourceStack) {
        self.inner.load(stack);
    }

    fn update(&mut self, _wrap: &mut WrapState) {
        self.inner.update();
    }

    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent) -> bool {
        event
            .reverse_map_position(self.matrix)
            .map(|e| self.inner.input(&e))
            .unwrap_or(false)
    }

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        let (child_size, changed) = self.inner.size();
        self.child_layout_size = child_size;
        (child_size.expand_width().expand_height(), changed)
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
        let child_size = self.child_layout_size.layout_one(size);
        let offset = (size.bottom_right() - child_size.bottom_right()) * 0.5;
        self.matrix = Matrix::translate(offset);
        self.inner.set_size(child_size);
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut skia::Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.inner.draw(canvas);
        canvas.restore();
    }
}
