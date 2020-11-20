use std::ffi::CString;

use stacks::{
    game::Builder,
    framework::{Framework, components::{
        Component, Transform, Parallax, Composite, Text, Font, FontStyle,
        shapes::{Rect, Throbber},
    }},
};
use skulpin_renderer::{skia_safe, PresentMode};

fn main() {
    let root = Parallax::new(Transform {
        inner: Composite::<Box<dyn Component + Send>> {
            inner: vec![
                Box::new(Rect::new(
                    (200.0, 100.0),
                    skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None),
                )),
                Box::new(Transform {
                    inner: Text {
                        text: "1234".into(),
                        font: Font::Default,
                        style: FontStyle::Bold,
                        paint: skia_safe::Paint::new(
                            skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0),
                            None,
                        ),
                    },
                    matrix: skia_safe::Matrix::translate((0.0, 120.0)),
                }),
                Box::new(Transform {
                    inner: Throbber::new(64.0, {
                        let mut p = skia_safe::Paint::new(
                            skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0),
                            None,
                        );
                        p.set_stroke_width(12.0);
                        p.set_anti_alias(true);
                        p.set_style(skia_safe::PaintStyle::Stroke);
                        p
                    }),
                    matrix: skia_safe::Matrix::translate((120.0, 240.0)),
                }),
            ],
        },
        matrix: skia_safe::Matrix::translate((120.0, 120.0)),
    });

    Builder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(Framework::new(root))
}
