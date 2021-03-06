use crate::prelude::*;

pub struct SizeFillContainer<T: Widget + ?Sized> {
    child: Wrap<T>,
    size: Size,
    child_size: LayoutSize,
    target_size: Option<Size>,
    matrix: Matrix,
}

impl<T: Widget + ?Sized> SizeFillContainer<T> {
    pub fn new(child: Wrap<T>, target_size: Option<Size>) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            child,
            size: Size::default(),
            child_size: LayoutSize::ZERO,
            target_size,
            matrix: Matrix::default(),
        }
        .into()
    }
}

impl<T: Widget + ?Sized> Widget for SizeFillContainer<T> {
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
        let (child_size, changed) = self.child.size();
        self.child_size = child_size;
        (LayoutSize::ZERO.expand_width().expand_height(), changed)
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.size = size;
        let target_size = self.target_size.unwrap_or_else(|| {
            (self.child_size.width.min, self.child_size.height.min).into()
        });
        let scale = (size.width / target_size.width)
            .min(size.height / target_size.height);
        self.matrix = Matrix::scale((scale, scale));
        let child_max_size = size / scale;
        self.child
            .set_size(self.child_size.layout_one(child_max_size));
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut skia::Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.child.draw(canvas);
        canvas.restore();
    }
}
