use skia::{Matrix, Paint};
use stacks::framework::{
    widgets::{
        audio::{Audio, AudioPlayer},
        layout::{ContainerSize, FullscreenContainer, SizeFillContainer, VContainer},
        shapes::{Rect, Throbber},
        Transform,
    },
    Framework,
};
use stacks::prelude::*;

fn main() {
    Framework::run("Stacks", || {
        let root = VContainer::new(
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
                        LayoutSize::min(100.0, 50.0),
                        Paint::new_color4f(0.0, 0.0, 1.0, 0.5).anti_alias(),
                    ),
                    Matrix::scale((1.5, 1.7)),
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
                    "resources/sound.ogg",
                    LayoutSize::min(500.0, 200.0).expand_width(),
                    Paint::new_color4f(0.8, 0.8, 1.0, 1.0).anti_alias(),
                    Paint::new_color4f(0.7, 0.7, 0.9, 0.7).anti_alias(),
                    Paint::new_color4f(1.0, 1.0, 1.0, 0.4).anti_alias(),
                )
                .expect("Failed to create AudioPlayer")
                .boxed(),
            ],
            ContainerSize::min(1366.0, 768.0)
                .expand_width()
                .expand_height(),
        );
        let root = FullscreenContainer::new(SizeFillContainer::new(root));
        Audio::new(root).expect("Failed to open audio")
    })
    .expect("Failed to run game");
}
