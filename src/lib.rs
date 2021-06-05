pub mod framework;
pub mod game;
pub mod utils;

pub use skia_safe as skia;

pub mod prelude {
    pub use crate::framework::resource::{
        ResourceHoster, ResourceStack, ResourceUsage, ResourceUsageMut,
        ResourceUser,
    };
    pub use crate::framework::widgets::{
        LayoutDimension, LayoutSize, Widget, WidgetState, Wrap, Wrappable,
    };
    pub use crate::framework::FrameworkState;
    pub use crate::game::{InputEvent, ScrollAmount, State};
    pub use crate::utils::*;
    pub use glutin::event::{MouseButton, VirtualKeyCode as Keycode};
    pub use skia::{
        scalar, Canvas, Contains, Matrix, Paint, Rect, Size, Vector,
    };
    pub use skia_safe as skia;
    pub use soloud;
}
