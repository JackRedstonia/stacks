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
    // let root = components::Transform {
    //     inner: components::shapes::Rect {
    //         rect: skia_safe::Rect {
    //             left: 0.0,
    //             top: 0.0,
    //             right: 100.0,
    //             bottom: 100.0,
    //         },
    //         paint: skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None),
    //     },
    //     matrix: skia_safe::Matrix::translate((200.0, 200.0)),
    // };

    let root = components::Composite {
        inner: (0..20000)
            .map(|f| {
                let wh = 200;
                let q = (f % wh) as f32 * 6.0;
                let p = (f / wh) as f32 * 6.0;
                components::Transform {
                    inner: components::shapes::Rect {
                        rect: skia_safe::Rect {
                            left: 0.0,
                            top: 0.0,
                            right: 5.0,
                            bottom: 5.0,
                        },
                        paint: skia_safe::Paint::new(
                            skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0),
                            None,
                        ),
                    },
                    matrix: skia_safe::Matrix::translate((q, p)),
                }
            })
            .collect(),
    };

    ApplicationBuilder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(Stacks::new(root))
}
