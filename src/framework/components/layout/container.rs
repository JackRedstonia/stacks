use super::super::{Component, LayoutDimension, LayoutSize};
use crate::skia::{scalar, Size, Vector};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ContainerSize {
    pub width: ContainerDimension,
    pub height: ContainerDimension,
}

impl ContainerSize {
    pub const ZERO: Self = Self {
        width: ContainerDimension::ZERO,
        height: ContainerDimension::ZERO,
    };

    pub fn min(width: scalar, height: scalar) -> Self {
        Self {
            width: ContainerDimension::min(width),
            height: ContainerDimension::min(height),
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
}

impl From<LayoutSize> for ContainerSize {
    fn from(s: LayoutSize) -> Self {
        Self {
            width: s.width.into(),
            height: s.height.into(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ContainerDimension {
    pub min: Option<scalar>,
    pub expand: Option<scalar>,
}

impl ContainerDimension {
    pub const ZERO: Self = Self {
        min: None,
        expand: None,
    };

    pub fn min(min: scalar) -> Self {
        Self {
            min: Some(min),
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
}

impl From<LayoutDimension> for ContainerDimension {
    fn from(d: LayoutDimension) -> Self {
        Self {
            min: Some(d.min),
            expand: d.expand,
        }
    }
}

pub struct ContainerComponent<T: Component> {
    pub inner: T,
    pub layout_size: LayoutSize,
    pub size: Size,
    pub position: Vector,
}

impl<T: Component> ContainerComponent<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            layout_size: LayoutSize::ZERO,
            size: Size::new_empty(),
            position: (0.0, 0.0).into(),
        }
    }
}
