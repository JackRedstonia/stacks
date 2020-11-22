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

use crate::game::{Canvas, InputEvent, InputState, TimeState};
use crate::skia::{scalar, Matrix, Rect, Size, Vector};

pub trait Component {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState);
    fn input(
        &mut self,
        input_state: &InputState,
        time_state: &TimeState,
        event: &InputEvent,
        size: Size,
    ) -> bool;

    fn size(&mut self, input_state: &InputState, time_state: &TimeState) -> LayoutSize;
    fn draw(
        &mut self,
        input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut Canvas,
        size: Size,
    );
}

impl Component for Box<dyn Component + Send> {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState) {
        self.as_mut().update(input_state, time_state);
    }

    fn input(
        &mut self,
        input_state: &InputState,
        time_state: &TimeState,
        event: &InputEvent,
        size: Size,
    ) -> bool {
        self.as_mut().input(input_state, time_state, event, size)
    }

    fn size(&mut self, input_state: &InputState, time_state: &TimeState) -> LayoutSize {
        self.as_mut().size(input_state, time_state)
    }

    fn draw(
        &mut self,
        input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut Canvas,
        size: Size,
    ) {
        self.as_mut().draw(input_state, time_state, canvas, size);
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

    pub fn layout_one(&self, space: scalar) -> scalar {
        if self.expand.is_some() {
            space
        } else {
            self.min
        }
    }
}
