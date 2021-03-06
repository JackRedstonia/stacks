use crate::prelude::*;

pub struct MarginContainer<T: Widget + ?Sized> {
    child: Wrap<T>,
    pub margin: Margin,
    size: Size,
    child_layout_size: LayoutSize,
    matrix: Matrix,
}

impl<T: Widget + ?Sized> MarginContainer<T> {
    pub fn new(child: Wrap<T>, margin: Margin) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            child,
            margin,
            child_layout_size: LayoutSize::ZERO,
            matrix: Matrix::default(),
            size: Size::default(),
        }
        .into()
    }
}

impl<T: Widget + ?Sized> Widget for MarginContainer<T> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.child.load(stack);
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.child.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        event
            .reverse_map_position(self.matrix)
            .map(|e| self.child.input(&e))
            .unwrap_or(false)
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let (mut child_size, changed) = self.child.size();
        self.child_layout_size = child_size;
        let margin_size = self.margin.size();
        child_size.width.min += margin_size.width;
        child_size.height.min += margin_size.height;
        (child_size, changed)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
        self.matrix = Matrix::translate((self.margin.left, self.margin.top));
        let child_size =
            size.bottom_right() - self.margin.size().bottom_right();
        self.child.set_size(Size::new(child_size.x, child_size.y));
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut skia::Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.child.draw(canvas);
        canvas.restore();
    }
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Margin {
    top: scalar,
    left: scalar,
    bottom: scalar,
    right: scalar,
}

impl Margin {
    pub const fn all(margin: scalar) -> Self {
        Self {
            top: margin,
            left: margin,
            bottom: margin,
            right: margin,
        }
    }

    pub const fn horizontal(margin: scalar) -> Self {
        Self::all(0.0).with_horizontal(margin)
    }

    pub const fn vertical(margin: scalar) -> Self {
        Self::all(0.0).with_vertical(margin)
    }

    pub const fn top(margin: scalar) -> Self {
        Self::all(0.0).with_top(margin)
    }

    pub const fn bottom(margin: scalar) -> Self {
        Self::all(0.0).with_bottom(margin)
    }

    pub const fn left(margin: scalar) -> Self {
        Self::all(0.0).with_left(margin)
    }

    pub const fn right(margin: scalar) -> Self {
        Self::all(0.0).with_right(margin)
    }

    pub const fn with_horizontal(self, margin: scalar) -> Self {
        Self {
            left: margin,
            right: margin,
            ..self
        }
    }

    pub const fn with_vertical(self, margin: scalar) -> Self {
        Self {
            top: margin,
            bottom: margin,
            ..self
        }
    }

    pub const fn with_top(self, margin: scalar) -> Self {
        Self {
            top: margin,
            ..self
        }
    }

    pub const fn with_bottom(self, margin: scalar) -> Self {
        Self {
            bottom: margin,
            ..self
        }
    }

    pub const fn with_left(self, margin: scalar) -> Self {
        Self {
            left: margin,
            ..self
        }
    }

    pub const fn with_right(self, margin: scalar) -> Self {
        Self {
            right: margin,
            ..self
        }
    }

    pub fn size(self) -> Size {
        Size::new(self.right + self.left, self.top + self.bottom)
    }
}
