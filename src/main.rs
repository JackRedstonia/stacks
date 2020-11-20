#![feature(duration_zero)]
#![feature(duration_constants)]

mod application;
mod canvas;
mod components;
mod stacks;

use std::ffi::CString;

use application::ApplicationBuilder;
use skulpin_renderer::{skia_safe, PresentMode};

use stacks::Stacks;

fn main() {
    let root = components::Transform {
        inner: components::Composite::<Box<dyn components::Component + Send>> {
            inner: vec![
                Box::new(components::shapes::Rect::new((200.0, 100.0), {
                    let mut p = skia_safe::Paint::new(
                        skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0),
                        None,
                    );
                    p.set_anti_alias(true);
                    p
                })),
                Box::new(components::Transform {
                    inner: components::Text {
                        text: "1234".into(),
                        font: components::Font::Default,
                        style: components::FontStyle::Regular,
                        paint: {
                            let mut p = skia_safe::Paint::new(
                                skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0),
                                None,
                            );
                            p.set_anti_alias(true);
                            p
                        }
                    },
                    matrix: skia_safe::Matrix::translate((0.0, 120.0)),
                })
            ],
        },
        matrix: skia_safe::Matrix::translate((120.0, 120.0)),
    };

    ApplicationBuilder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(Stacks::new(root))
}
