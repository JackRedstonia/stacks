#![feature(duration_zero)]
#![feature(duration_constants)]

pub mod framework;
pub mod game;
pub mod utils;

pub use skulpin_renderer::skia_safe as skia;

pub mod prelude {
    pub use crate::game;
    pub use crate::framework::widgets::{
        Widget, Wrappable, Wrap, WrapState, LayoutSize, LayoutDimension,
    };
    pub use skulpin_renderer::skia_safe as skia;
    pub use crate::utils::*;
}
