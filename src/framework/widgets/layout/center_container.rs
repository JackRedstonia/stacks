use crate::prelude::*;

pub struct CenterContainer<T: Widget + ?Sized> {
    child: Wrap<T>,
    size: Size,
    child_layout_size: LayoutSize,
    matrix: Matrix,
}

impl<T: Widget + ?Sized> CenterContainer<T> {
    pub fn new(child: Wrap<T>) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            child,
            child_layout_size: LayoutSize::ZERO,
            matrix: Matrix::default(),
            size: Size::default(),
        }
        .into()
    }
}

impl<T: Widget + ?Sized> Widget for CenterContainer<T> {
    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack) {
        self.child.load(stack);
    }

    fn update(&mut self, state: &mut WidgetState) {
        self.child.update();
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        event
            .reverse_map_position(self.matrix)
            .map(|e| self.child.input(&e))
            .unwrap_or(false)
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        let s = self.child.size();
        self.child_layout_size = s.0;
        s
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.size = size;
        let child_size = self.child_layout_size.layout_one(size);
        let offset = (size.bottom_right() - child_size.bottom_right()) * 0.5;
        self.matrix = Matrix::translate(offset);
        self.child.set_size(child_size);
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut skia::Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.child.draw(canvas);
        canvas.restore();
    }
}
