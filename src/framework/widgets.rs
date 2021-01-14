mod audio_player;
pub mod layout;
mod parallax;
pub mod shapes;
mod text;
mod transform;

pub use audio_player::AudioPlayer;
pub use parallax::Parallax;
pub use text::{Font, FontStyle, Text};
pub use transform::Transform;

use crate::game::{InputEvent, ID};
use crate::skia::{scalar, Canvas, Matrix, Rect, Size, Vector};

#[allow(unused_variables)]
pub trait Widget: 'static + Send {
    fn update(&mut self, wrap: &mut WrapState) {}

    fn input(&mut self, wrap: &mut WrapState, event: &InputEvent) -> bool {
        false
    }

    fn hover(&mut self, wrap: &mut WrapState) {}

    fn hover_lost(&mut self, wrap: &mut WrapState) {}

    fn size(&mut self, wrap: &mut WrapState) -> (LayoutSize, bool) {
        (LayoutSize::ZERO, false)
    }

    fn set_size(&mut self, wrap: &mut WrapState, size: Size) {}

    fn draw(&mut self, wrap: &mut WrapState, canvas: &mut Canvas) {}

    fn get(&mut self, wrap: &mut WrapState, id: ID) -> Option<(&mut dyn Widget, &mut WrapState)> {
        None
    }
}

impl Widget for Box<dyn Widget> {
    fn update(&mut self, wrap: &mut WrapState) {
        self.as_mut().update(wrap);
    }

    fn input(&mut self, wrap: &mut WrapState, event: &InputEvent) -> bool {
        self.as_mut().input(wrap, event)
    }

    fn hover(&mut self, wrap: &mut WrapState) {
        self.as_mut().hover(wrap);
    }

    fn hover_lost(&mut self, wrap: &mut WrapState) {
        self.as_mut().hover_lost(wrap);
    }

    fn size(&mut self, wrap: &mut WrapState) -> (LayoutSize, bool) {
        self.as_mut().size(wrap)
    }

    fn set_size(&mut self, wrap: &mut WrapState, size: Size) {
        self.as_mut().set_size(wrap, size);
    }

    fn draw(&mut self, wrap: &mut WrapState, canvas: &mut Canvas) {
        self.as_mut().draw(wrap, canvas);
    }

    fn get(&mut self, wrap: &mut WrapState, id: ID) -> Option<(&mut dyn Widget, &mut WrapState)> {
        self.as_mut().get(wrap, id)
    }
}

pub struct Wrap<T: Widget> {
    pub inner: T,
    pub state: WrapState,
}

impl<T: Widget> Wrap<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            state: WrapState::new(),
        }
    }

    pub fn update(&mut self) {
        self.inner.update(&mut self.state);
    }

    pub fn input(&mut self, event: &InputEvent) -> bool {
        let b = self.inner.input(&mut self.state, event);
        if matches!(event, InputEvent::MouseMove(_)) {
            if self.state.was_hovered != b {
                self.state.was_hovered = b;
                if b {
                    self.inner.hover(&mut self.state);
                } else {
                    self.inner.hover_lost(&mut self.state);
                }
            }
        }
        b
    }

    pub fn size(&mut self) -> (LayoutSize, bool) {
        self.inner.size(&mut self.state)
    }

    pub fn set_size(&mut self, size: Size) {
        self.inner.set_size(&mut self.state, size);
    }

    pub fn draw(&mut self, canvas: &mut Canvas) {
        self.inner.draw(&mut self.state, canvas);
    }

    pub fn get(&mut self, id: ID) -> Option<(&mut dyn Widget, &mut WrapState)> {
        if self.state.id == id {
            Some((&mut self.inner, &mut self.state))
        } else {
            self.inner.get(&mut self.state, id)
        }
    }
}

impl<T: Widget> From<T> for Wrap<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

pub trait Wrappable<T: Widget> {
    fn wrap(self) -> Wrap<T>;
    fn boxed(self) -> Box<dyn Widget>;
}

impl<T: 'static + Widget> Wrappable<T> for T {
    fn wrap(self) -> Wrap<T> {
        Wrap::new(self)
    }

    fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

#[derive(Debug)]
pub struct WrapState {
    id: ID,
    was_hovered: bool,
}

impl WrapState {
    pub fn new() -> Self {
        Self {
            id: ID::next(),
            was_hovered: false,
        }
    }

    pub fn id(&self) -> &ID {
        &self.id
    }

    pub fn is_hovered(&self) -> bool {
        self.was_hovered
    }
}

impl Default for WrapState {
    fn default() -> Self {
        Self::new()
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
    /// Normally ignored by containers and other UI-building widgets,
    /// and is only respected when this is not part of an UI.
    pub size: scalar,

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
