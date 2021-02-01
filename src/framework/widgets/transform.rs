use crate::prelude::*;

pub struct Transform<T: Widget> {
    pub inner: Wrap<T>,
    pub matrix: Matrix,
}

impl<T: Widget> Transform<T> {
    pub fn new(inner: Wrap<T>, matrix: Matrix) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            inner: inner.into(),
            matrix,
        }
        .into()
    }
}

impl<T: Widget> Widget for Transform<T> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.inner.load(stack);
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.inner.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        event
            .reverse_map_position(self.matrix)
            .map_or(false, |event| self.inner.input(&event))
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let s = self.inner.size();
        (s.0.map(self.matrix), s.1)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        if let Some(m) = self.matrix.invert() {
            let (rect, _) = m.map_rect(Rect::from_size(size));
            self.inner.set_size(rect.size());
        }
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.inner.draw(canvas);
        canvas.restore();
    }
}
