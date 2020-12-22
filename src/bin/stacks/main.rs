use std::ffi::CString;

use skulpin_renderer::PresentMode;
use stacks::{
    framework::{
        widgets::{
            Wrappable,
            layout::{ContainerSize, VContainer},
            shapes::{Rect, Throbber},
            LayoutDimension, LayoutSize, Transform, Widget,
            Parallax,
        },
        Framework,
    },
    game::Builder,
    skia,
};

fn main() {
    let root = Parallax::new(VContainer::new(
        vec![
            Rect::new(
                LayoutSize::min(200.0, 100.0).expand_width().expand_height(),
                skia::Paint::new(skia::Color4f::new(0.0, 1.0, 0.0, 1.0), None),
                false,
            ).boxed().wrap(),
            Rect::new(
                LayoutSize::min(100.0, 100.0).expand_height_by(3.0),
                skia::Paint::new(skia::Color4f::new(1.0, 0.0, 0.0, 1.0), None),
                false,
            ).boxed().wrap(),
            Transform::new(
                Rect::new(
                    LayoutSize::min(50.0, 100.0),
                    skia::Paint::new(skia::Color4f::new(0.0, 0.0, 1.0, 1.0), None),
                    false,
                ).wrap(),
                skia::Matrix::translate((20.0, 20.0)),
            ).boxed().wrap(),
            Throbber::new(
                LayoutDimension::min(100.0),
                {
                    let mut p = skia::Paint::new(skia::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
                    p.set_stroke_width(12.0);
                    p.set_anti_alias(true);
                    p.set_stroke(true);
                    p
                },
                false,
            ).boxed().wrap(),
        ],
        ContainerSize::ZERO.expand_height().expand_width(),
    ).wrap()).wrap();

    Builder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(Framework::new(root))
}
