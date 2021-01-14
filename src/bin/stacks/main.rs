use std::ffi::CString;

use game::Builder;
use skia::{Matrix, Paint};
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
                        Paint::new_color4f(0.2, 0.8, 0.0, 0.3),
                    )
                    .boxed(),
                    Rect::new(
                        LayoutSize::min(100.0, 100.0).expand_height_by(3.0),
                        Paint::new_color4f(0.7, 0.1, 0.2, 0.3).anti_alias(),
                    )
                    .boxed(),
                    Transform::new(
                        Rect::new(
                            LayoutSize::min(50.0, 100.0),
                            Paint::new_color4f(0.0, 0.0, 1.0, 1.0).anti_alias(),
                        ),
                        Matrix::translate((20.0, 20.0)),
                    )
                    .boxed(),
                    Throbber::new(
                        LayoutDimension::min(100.0),
                        Paint::new_color4f(0.0, 1.0, 0.0, 1.0)
                            .with_stroke_width(12.0)
                            .anti_alias()
                            .stroke(),
                    )
                    .boxed(),
                    AudioPlayer::new(
                        LayoutSize::min(500.0, 100.0).expand_width(),
                        Paint::new_color4f(0.8, 0.8, 1.0, 1.0).anti_alias(),
                        Paint::new_color4f(0.7, 0.7, 0.9, 0.7).anti_alias(),
                    )
                    .boxed(),
                ],
                ContainerSize::ZERO.expand_height().expand_width(),
            ));
            Framework::new(root)
        })
}
