use crate::skia::{scalar, Matrix, Rect, Size, Vector};

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct LayoutSize {
    pub width: LayoutDimension,
    pub height: LayoutDimension,
}

impl LayoutSize {
    pub const ZERO: Self = Self {
        width: LayoutDimension::ZERO,
        height: LayoutDimension::ZERO,
    };

    pub fn min(width: scalar, height: scalar) -> Self {
        Self {
            width: LayoutDimension::min(width),
            height: LayoutDimension::min(height),
        }
    }

    pub fn with_expand_from(self, other: &Self) -> Self {
        self.with_expand_width(other.width.expand)
            .with_expand_height(other.height.expand)
    }

    pub fn with_expand_width(mut self, expand: Option<scalar>) -> Self {
        self.width.expand = expand;
        self
    }

    pub fn expand_width_by(mut self, expand: scalar) -> Self {
        self.width.expand = Some(expand);
        self
    }

    pub fn expand_width(mut self) -> Self {
        self.width.expand = Some(1.0);
        self
    }

    pub fn no_expand_width(mut self) -> Self {
        self.width.expand = None;
        self
    }

    pub fn with_expand_height(mut self, expand: Option<scalar>) -> Self {
        self.height.expand = expand;
        self
    }

    pub fn expand_height_by(mut self, expand: scalar) -> Self {
        self.height.expand = Some(expand);
        self
    }

    pub fn expand_height(mut self) -> Self {
        self.height.expand = Some(1.0);
        self
    }

    pub fn no_expand_height(mut self) -> Self {
        self.height.expand = None;
        self
    }

    pub fn get_min(&self) -> Size {
        Size::new(self.width.min, self.height.min)
    }

    pub fn layout_one(&self, size: Size) -> Size {
        Size::new(
            self.width.layout_one(size.width),
            self.height.layout_one(size.height),
        )
    }

    pub fn map(&self, matrix: Matrix) -> Self {
        let min = Self::map_size(self.width.min, self.height.min, matrix);
        let expand_width = self
            .width
            .expand
            .map(|x| matrix.map_vector(Vector::new(x, 0.0)).x.abs());
        let expand_height = self
            .height
            .expand
            .map(|y| matrix.map_vector(Vector::new(0.0, y)).y.abs());
        Self {
            width: LayoutDimension {
                min: min.width.abs(),
                expand: expand_width,
            },
            height: LayoutDimension {
                min: min.height.abs(),
                expand: expand_height,
            },
        }
    }

    fn map_size(width: scalar, height: scalar, matrix: Matrix) -> Size {
        matrix.map_rect(Rect::from_wh(width, height)).0.size()
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct LayoutDimension {
    /// The minimum number of logical pixels this dimension should receive.
    /// May not always be respected by containers, for example when they run out
    /// of space.
    pub min: scalar,

    /// The expansion factor in this dimension.
    /// A Some(x) expresses this widget in this dimension should take up as
    /// much space as it is allowed, with an expansion factor of x.
    /// A None expresses this widget should take up the minimum space,
    /// expressed in the [min](Self::min) field.
    pub expand: Option<scalar>,
}

impl LayoutDimension {
    pub const ZERO: Self = Self {
        min: 0.0,
        expand: None,
    };

    pub fn min(min: scalar) -> Self {
        Self { min, expand: None }
    }

    pub fn expand_by(mut self, expand: scalar) -> Self {
        self.expand = Some(expand);
        self
    }

    pub fn expand(mut self) -> Self {
        self.expand = Some(1.0);
        self
    }

    pub fn no_expand(mut self) -> Self {
        self.expand = None;
        self
    }

    pub fn layout_one(&self, space: scalar) -> scalar {
        if self.expand.is_some() {
            space.max(self.min)
        } else {
            self.min
        }
    }
}
