use crate::prelude::*;
use game::InputEvent;
use skia::{scalar, Matrix, Size};

pub struct MarginContainer<T: Widget> {
    pub margin: Margin,
    size: Size,
    child_layout_size: LayoutSize,
    matrix: Matrix,
    inner: Wrap<T>,
}

impl<T: Widget> MarginContainer<T> {
    pub fn new(inner: impl Into<Wrap<T>>, margin: Margin) -> Self {
        FrameworkState::request_load();
        Self {
            margin,
            inner: inner.into(),
            child_layout_size: LayoutSize::ZERO,
            matrix: Matrix::default(),
            size: Size::default(),
        }
    }
}

impl<T: Widget> Widget for MarginContainer<T> {
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
        (LayoutSize::ZERO.expand_width().expand_height(), changed)
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
        let child_size = size.bottom_right() - self.margin.size().bottom_right();
        self.matrix = Matrix::translate((self.margin.left, self.margin.top));
        self.inner.set_size(Size::new(child_size.x, child_size.y));
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut skia::Canvas) {
        canvas.save();
        canvas.concat(&self.matrix);
        self.inner.draw(canvas);
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
