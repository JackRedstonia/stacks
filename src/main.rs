#![feature(duration_zero)]

mod application;
mod canvas;
mod components;
mod stacks;

use std::ffi::CString;

use application::ApplicationBuilder;
use skulpin_renderer::{skia_safe, PresentMode};

use stacks::Stacks;

fn main() {
    ApplicationBuilder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(Stacks::new(components::shapes::Rect {
            rect: skia_safe::Rect {
                left: 0.0,
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
            },
            paint: skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None),
        }))
}
