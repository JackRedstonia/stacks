use super::super::{LayoutDimension, LayoutSize, Widget, Wrap};
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

pub struct ContainerState {
    pub layout_size: LayoutSize,
    pub position: Vector,
    size: Size,
    changed: bool,
    children_changed: bool,
}

impl ContainerState {
    pub fn new() -> Self {
        Self {
            layout_size: LayoutSize::ZERO,
            position: (0.0, 0.0).into(),
            size: Size::new_empty(),
            changed: true,
            children_changed: true,
        }
    }

    pub fn size(
        &mut self,
        widget: &mut Wrap<dyn Widget>,
    ) -> (LayoutSize, bool, bool) {
        let (s, c) = widget.size();
        self.changed = self.layout_size != s;
        self.layout_size = s;
        self.children_changed = c;

        (s, self.changed, c)
    }

    pub fn maybe_set_size(
        &mut self,
        widget: &mut Wrap<dyn Widget>,
        size: Size,
    ) {
        if self.changed || self.children_changed || size != self.size {
            self.size = size;
            widget.set_size(size);
        }
    }
}
