use std::ffi::CString;

use skulpin_renderer::PresentMode;
use stacks::{
    framework::{
        components::{
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
    let root = Parallax::new(Rect::new(
        LayoutSize::min(200.0, 100.0),
        skia::Paint::new(skia::Color4f::new(0.0, 1.0, 0.0, 1.0), None),
        false,
    ));
    // let root = Parallax::new(Transform {
    //     inner: Composite::<Box<dyn Component + Send>> {
    //         inner: vec![
    //             Box::new(Rect::new(
    //                 (200.0, 100.0),
    //                 skia::Paint::new(skia::Color4f::new(0.0, 1.0, 0.0, 1.0), None),
    //             )),
    //             Box::new(Transform {
    //                 inner: Text {
    //                     text: "1234".into(),
    //                     font: Font::Default,
    //                     style: FontStyle::Bold,
    //                     paint: skia::Paint::new(
    //                         skia::Color4f::new(0.0, 1.0, 0.0, 1.0),
    //                         None,
    //                     ),
    //                 },
    //                 matrix: skia::Matrix::translate((0.0, 120.0)),
    //             }),
    //             Box::new(Transform {
    //                 inner: Throbber::new(64.0, {
    //                     let mut p = skia::Paint::new(
    //                         skia::Color4f::new(0.0, 1.0, 0.0, 1.0),
    //                         None,
    //                     );
    //                     p.set_stroke_width(12.0);
    //                     p.set_anti_alias(true);
    //                     p.set_style(skia::PaintStyle::Stroke);
    //                     p
    //                 }),
    //                 matrix: skia::Matrix::translate((120.0, 240.0)),
    //             }),
    //         ],
    //     },
    //     matrix: skia::Matrix::translate((120.0, 120.0)),
    // });

    Builder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(Framework::new(root))
}
