use std::ffi::CString;

use game::Builder;
use skia::{Color4f, Matrix, Paint};
use stacks::framework::{
    widgets::{
        layout::{ContainerSize, VContainer},
        shapes::{Rect, Throbber},
        AudioPlayer, Parallax, Transform,
    },
    Framework,
};
use stacks::prelude::*;

fn main() {
    Builder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .run(|| {
            let root = Parallax::new(VContainer::new(
                vec![
                    Rect::new(
                        LayoutSize::min(200.0, 100.0).expand_width().expand_height(),
                        Paint::new(Color4f::new(0.2, 0.8, 0.0, 0.3), None).with_anti_alias(true),
                        false,
                    )
                    .boxed(),
                    Rect::new(
                        LayoutSize::min(100.0, 100.0).expand_height_by(3.0),
                        Paint::new(Color4f::new(0.7, 0.1, 0.2, 0.3), None).with_anti_alias(true),
                        false,
                    )
                    .boxed(),
                    Transform::new(
                        Rect::new(
                            LayoutSize::min(50.0, 100.0),
                            Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None)
                                .with_anti_alias(true),
                            false,
                        ),
                        Matrix::translate((20.0, 20.0)),
                    )
                    .boxed(),
                    Throbber::new(
                        LayoutDimension::min(100.0),
                        {
                            let mut p = Paint::new(Color4f::new(0.0, 1.0, 0.0, 1.0), None)
                                .with_anti_alias(true);
                            p.set_stroke_width(12.0);
                            p.set_stroke(true);
                            p
                        },
                        false,
                    )
                    .boxed(),
                    AudioPlayer::new(
                        LayoutSize::min(500.0, 100.0).expand_width(),
                        Paint::new(Color4f::new(0.8, 0.8, 1.0, 1.0), None).with_anti_alias(true),
                        Paint::new(Color4f::new(0.7, 0.7, 0.9, 0.7), None).with_anti_alias(true),
                    )
                    .boxed(),
                ],
                ContainerSize::ZERO.expand_height().expand_width(),
            ));
            Framework::new(root)
        })
}
