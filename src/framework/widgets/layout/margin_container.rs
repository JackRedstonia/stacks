use crate::prelude::*;

pub struct MarginContainer {
    pub margin: Margin,
    size: Size,
    child_layout_size: LayoutSize,
    matrix: Matrix,
}

impl MarginContainer {
    pub fn new(margin: Margin) -> Wrap<Self> {
        Self {
            margin,
            child_layout_size: LayoutSize::ZERO,
            matrix: Matrix::default(),
            size: Size::default(),
        }
        .into()
    }
}

impl Widget for MarginContainer {
    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        state
            .child()
            .map(|child| {
                event
                    .reverse_map_position(self.matrix)
                    .map(|e| child.input(&e))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        state
            .child()
            .map(|child| {
                let (mut child_size, changed) = child.size();
                self.child_layout_size = child_size;
                let margin_size = self.margin.size();
                child_size.width.min += margin_size.width;
                child_size.height.min += margin_size.height;
                (child_size, changed)
            })
            .unwrap_or_default()
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.size = size;
        self.matrix = Matrix::translate((self.margin.left, self.margin.top));
        if let Some(child) = state.child() {
            let child_size =
                size.bottom_right() - self.margin.size().bottom_right();
            child.set_size(Size::new(child_size.x, child_size.y));
        }
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut skia::Canvas) {
        if let Some(child) = state.child() {
            canvas.save();
            canvas.concat(&self.matrix);
            child.draw(canvas);
            canvas.restore();
        }
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
