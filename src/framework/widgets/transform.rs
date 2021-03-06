use crate::prelude::*;

pub struct Transform<T: Widget + ?Sized> {
    child: Wrap<T>,
    pub matrix: Matrix,
}

impl<T: Widget + ?Sized> Transform<T> {
    pub fn new(child: Wrap<T>, matrix: Matrix) -> Wrap<Self> {
        FrameworkState::request_load();
        Self { child, matrix }.into()
    }
}

impl<T: Widget + ?Sized> Widget for Transform<T> {
    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack) {
        self.child.load(stack);
    }

    fn update(&mut self, state: &mut WidgetState) {
        self.child.update();
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        event
            .reverse_map_position(self.matrix)
            .map_or(false, |event| self.child.input(&event))
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        let s = self.child.size();
        (s.0.map(self.matrix), s.1)
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        if let Some(m) = self.matrix.invert() {
            let (rect, _) = m.map_rect(Rect::from_size(size));
            self.child.set_size(rect.size());
        }
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.child.draw(canvas);
        canvas.restore();
    }
}
