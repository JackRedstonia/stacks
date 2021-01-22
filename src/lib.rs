#![feature(duration_zero)]
#![feature(duration_constants)]

pub mod framework;
pub mod game;
pub mod utils;

pub use skulpin_renderer::skia_safe as skia;

pub mod prelude {
    pub use crate::framework::{
        audio,
        resource::{ResourceHoster, ResourceStack, ResourceUser},
        widgets::{LayoutDimension, LayoutSize, Widget, Wrap, WrapState, Wrappable},
        FrameworkState,
    };
    pub use crate::game;
    pub use crate::utils::*;
    pub use skulpin_renderer::skia_safe as skia;
    pub use skulpin_renderer_sdl2::sdl2::{keyboard::Keycode, mouse::MouseButton};
    pub use soloud;
}
