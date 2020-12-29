use std::ffi::CString;

use skulpin_renderer::PresentMode;
use stacks::{
    framework::{
        widgets::{
            layout::{ContainerSize, VContainer},
            shapes::{Rect, Throbber},
            Font, FontStyle, LayoutDimension, LayoutSize, Text, Transform, Wrappable,
        },
        Framework,
    },
    game::Builder,
    skia::{Color4f, Matrix, Paint},
};

fn main() {
    Builder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(|| {
            let root = VContainer::new(
                vec![
                    Rect::new(
                        LayoutSize::min(200.0, 100.0).expand_width().expand_height(),
                        Paint::new(Color4f::new(0.2, 0.8, 0.0, 0.3), None),
                        false,
                    )
                    .boxed()
                    .wrap(),
                    Rect::new(
                        LayoutSize::min(100.0, 100.0).expand_height_by(3.0),
                        Paint::new(Color4f::new(0.7, 0.1, 0.2, 0.3), None),
                        false,
                    )
                    .boxed()
                    .wrap(),
                    Transform::new(
                        Rect::new(
                            LayoutSize::min(50.0, 100.0),
                            Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None),
                            false,
                        )
                        .wrap(),
                        Matrix::translate((20.0, 20.0)),
                    )
                    .boxed()
                    .wrap(),
                    Throbber::new(
                        LayoutDimension::min(100.0),
                        {
                            let mut p = Paint::new(Color4f::new(0.0, 1.0, 0.0, 1.0), None);
                            p.set_stroke_width(12.0);
                            p.set_anti_alias(true);
                            p.set_stroke(true);
                            p
                        },
                        false,
                    )
                    .boxed()
                    .wrap(),
                    Text::new(
                        LayoutSize::min(500.0, 100.0).expand_width(),
                        "lorem ipsum dolor sit amet.  いろはにほへと",
                        Font::Default,
                        FontStyle::Bold,
                        {
                            let mut p = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                            p.set_anti_alias(true);
                            p
                        },
                    )
                    .boxed()
                    .wrap(),
                ],
                ContainerSize::ZERO.expand_height().expand_width(),
            )
            .wrap();
            Framework::new(root)
        })
}
