use super::{LayoutSize, Widget, Wrap, WrapState, ID};
use crate::game::InputEvent;
use crate::skia;
use skia::{Canvas, Matrix, Rect, Size};

pub struct Transform<T: Widget> {
    pub inner: Wrap<T>,
    size: LayoutSize,
    pub matrix: Matrix,
}

impl<T: Widget> Transform<T> {
    pub fn new(inner: Wrap<T>, matrix: Matrix) -> Self {
        Self {
            inner,
            size: LayoutSize::ZERO,
            matrix,
        }
    }
}

impl<T: Widget> Widget for Transform<T> {
    fn update(&mut self, _wrap: &mut WrapState) {
        self.inner.update();
    }

    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent) -> bool {
        event
            .reverse_map_position(self.matrix)
            .map_or(false, |event| self.inner.input(&event))
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        self.size = self.inner.size();
        self.size.map(self.matrix)
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        if let Some(m) = self.matrix.invert() {
            let (rect, _) = m.map_rect(Rect::from_size(size));
            self.inner.set_size(rect.size());
        }
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.inner.draw(canvas);
        canvas.restore();
    }

    fn get(&mut self, _wrap: &mut WrapState, id: ID) -> Option<(&mut dyn Widget, &mut WrapState)> {
        self.inner.get(id)
    }
}
