use std::ffi::CString;

use skulpin_renderer::PresentMode;
use stacks::{
    framework::{
        components::{
            layout::HContainer,
            shapes::{Rect, Throbber},
            Component, Font, FontStyle, LayoutDimension, LayoutSize, Parallax,
            /*Composite,*/ Text, Transform,
        },
        Framework,
    },
    game::Builder,
    skia,
};

fn main() {
    let root = HContainer::<Box<dyn Component + Send>>::new(vec![
        Box::new(Rect::new(
            {
                let mut size = LayoutSize::min(200.0, 100.0);
                size.width.expand = Some(1.0);
                size
            },
            skia::Paint::new(skia::Color4f::new(0.0, 1.0, 0.0, 1.0), None),
            false,
        )),
        Box::new(Rect::new(
            {
                let mut size = LayoutSize::min(100.0, 100.0);
                // Uncomment to expand first rectangle
                size.width.expand = Some(3.0);
                size
            },
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
    ]);

    Builder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(Framework::new(root))
}
