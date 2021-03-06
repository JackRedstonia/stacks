pub mod framework;
pub mod game;
pub mod utils;

pub use skulpin_renderer::skia_safe as skia;

pub mod prelude {
    pub use crate::framework::resource::{
        ResourceHoster, ResourceStack, ResourceUsage, ResourceUsageMut,
        ResourceUser,
    };
    pub use crate::framework::widgets::{
        LayoutDimension, LayoutSize, Widget, WidgetState, Wrap, Wrappable,
    };
    pub use crate::framework::FrameworkState;
    pub use crate::game::{InputEvent, State};
    pub use crate::utils::*;
    pub use skia::{
        scalar, Canvas, Contains, Matrix, Paint, Rect, Size, Vector,
    };
    pub use skulpin_renderer::skia_safe as skia;
    pub use skulpin_renderer_sdl2::sdl2::{
        keyboard::Keycode, mouse::MouseButton,
    };
    pub use soloud;
}
