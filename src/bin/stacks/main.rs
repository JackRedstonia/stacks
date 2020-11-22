use std::ffi::CString;

use skulpin_renderer::PresentMode;
use stacks::{
    framework::{
        components::{
            layout::{VContainer, ContainerSize},
            shapes::{Rect, Throbber},
            Component, LayoutDimension, LayoutSize, Transform,
        },
        Framework,
    },
    game::Builder,
    skia,
};

fn main() {
    let root = VContainer::<Box<dyn Component + Send>>::new(vec![
        Box::new(Rect::new(
            LayoutSize::min(200.0, 100.0).expand_width().expand_height(),
            skia::Paint::new(skia::Color4f::new(0.0, 1.0, 0.0, 1.0), None),
            false,
        )),
        Box::new(Rect::new(
            LayoutSize::min(100.0, 100.0).expand_height_by(3.0),
            skia::Paint::new(skia::Color4f::new(1.0, 0.0, 0.0, 1.0), None),
            false,
        )),
        Box::new(Transform::new(
            Rect::new(
                LayoutSize::min(50.0, 100.0),
                skia::Paint::new(skia::Color4f::new(0.0, 0.0, 1.0, 1.0), None),
                false,
            ),
            skia::Matrix::translate((20.0, 20.0)),
        )),
        Box::new(Throbber::new(
            LayoutDimension::min(100.0),
            {
                let mut p = skia::Paint::new(skia::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
                p.set_stroke_width(12.0);
                p.set_anti_alias(true);
                p.set_stroke(true);
                p
            },
            false,
        )),
    ], ContainerSize::ZERO.expand_height().expand_width());

    Builder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(Framework::new(root))
}
