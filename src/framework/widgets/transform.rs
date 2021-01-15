use crate::prelude::*;
use game::InputEvent;
use skia::{Canvas, Matrix, Rect, Size};

pub struct Transform<T: Widget> {
    pub inner: Wrap<T>,
    pub matrix: Matrix,
}

impl<T: Widget> Transform<T> {
    pub fn new(inner: impl Into<Wrap<T>>, matrix: Matrix) -> Self {
        Self {
            inner: inner.into(),
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

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        let s = self.inner.size();
        (s.0.map(self.matrix), s.1)
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

    // fn get(&mut self, _wrap: &mut WrapState, id: ID) -> Option<(&mut dyn Widget, &mut WrapState)> {
    //     self.inner.get(id)
    // }
}
