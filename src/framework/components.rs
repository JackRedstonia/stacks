pub mod layout;
mod metrics;
mod parallax;
pub mod shapes;
mod text;
mod transform;

pub use metrics::Metrics;
pub use parallax::Parallax;
pub use text::{Font, FontStyle, Text};
pub use transform::Transform;

use crate::game::{Canvas, InputEvent};
use crate::skia::{scalar, Matrix, Rect, Size, Vector};

pub trait Component {
    fn update(&mut self) {}

    fn input(&mut self, _event: &InputEvent, _size: Size) -> bool {
        false
    }

    fn size(&mut self) -> LayoutSize {
        LayoutSize::ZERO
    }

    fn draw(&mut self, _canvas: &mut Canvas, _size: Size) {}
}

impl Component for Box<dyn Component + Send> {
    fn update(&mut self) {
        self.as_mut().update();
    }

    fn input(&mut self, event: &InputEvent, size: Size) -> bool {
        self.as_mut().input(event, size)
    }

    fn size(&mut self) -> LayoutSize {
        self.as_mut().size()
    }

    fn draw(&mut self, canvas: &mut Canvas, size: Size) {
        self.as_mut().draw(canvas, size);
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
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

    pub fn layout_one(&self, size: Size) -> Size {
        Size::new(
            self.width.layout_one(size.width),
            self.height.layout_one(size.height),
        )
    }

    pub fn map(&self, matrix: Matrix) -> Self {
        let min = Self::map_size(self.width.min, self.height.min, matrix);
        let size = Self::map_size(self.width.min, self.height.min, matrix);
        let expand_width = self
            .width
            .expand
            .map(|x| matrix.map_vector(Vector::new(x, 0.0)).x);
        let expand_height = self
            .height
            .expand
            .map(|y| matrix.map_vector(Vector::new(0.0, y)).y);
        Self {
            width: LayoutDimension {
                min: min.width.abs(),
                size: size.width.abs(),
                expand: expand_width,
            },
            height: LayoutDimension {
                min: min.height.abs(),
                size: size.height.abs(),
                expand: expand_height,
            },
        }
    }

    fn map_size(width: scalar, height: scalar, matrix: Matrix) -> Size {
        let r = matrix.map_rect(Rect::from_wh(width, height)).0;
        Size::new(r.right.abs(), r.bottom.abs())
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LayoutDimension {
    /// The minimum number of logical pixels this dimension should receive.
    /// May not always be respected by containers, for example when they run out
    /// of space.
    pub min: scalar,

    /// The normal number of logical pixels in this dimension.
    /// Normally ignored by containers and other UI-building components,
    /// and is only respected when this is not part of an UI.
    pub size: scalar,

    /// The expansion factor in this dimension.
    /// A Some(x) expresses this component in this dimension should take up as
    /// much space as it is allowed, with an expansion factor of x.
    /// A None expresses this component should take up the minimum space,
    /// expressed in the [min](Self::min) field.
    pub expand: Option<scalar>,
}

impl LayoutDimension {
    pub const ZERO: Self = Self {
        min: 0.0,
        size: 0.0,
        expand: None,
    };

    pub fn min(min: scalar) -> Self {
        Self {
            min,
            size: 0.0,
            expand: None,
        }
    }

    pub fn with_expand(mut self, expand: scalar) -> Self {
        self.expand = Some(expand);
        self
    }

    pub fn with_expand_one(mut self) -> Self {
        self.expand = Some(1.0);
        self
    }

    pub fn with_no_expand(mut self) -> Self {
        self.expand = None;
        self
    }

    pub fn layout_one(&self, space: scalar) -> scalar {
        if self.expand.is_some() {
            space
        } else {
            self.min
        }
    }
}
