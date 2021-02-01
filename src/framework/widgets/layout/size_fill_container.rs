use crate::prelude::*;

pub struct SizeFillContainer<T: Widget> {
    size: Size,
    child_size: LayoutSize,
    target_size: Size,
    matrix: Matrix,
    inner: Wrap<T>,
}

impl<T: Widget> SizeFillContainer<T> {
    pub fn new(inner: impl Into<Wrap<T>>, target_size: Size) -> Self {
        FrameworkState::request_load();
        Self {
            size: Size::default(),
            child_size: LayoutSize::ZERO,
            target_size,
            matrix: Matrix::default(),
            inner: inner.into(),
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
        self.child_size = child_size;
        (LayoutSize::ZERO.expand_width().expand_height(), changed)
    }

    fn set_size(&mut self, _wrap: &mut WidgetState, size: Size) {
        self.size = size;
        let scale = (size.width / self.target_size.width).min(size.height / self.target_size.height);
        self.matrix = Matrix::scale((scale, scale));
        let child_max_size = size / scale;
        self.inner
            .set_size(self.child_size.layout_one(child_max_size));
    }

    fn draw(&mut self, _wrap: &mut WidgetState, canvas: &mut skia::Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.inner.draw(canvas);
        canvas.restore();
    }
}
