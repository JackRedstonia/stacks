use stacks::framework::{
    widgets::{
        audio::{Audio, AudioPlayer},
        layout::{
            ContainerSize, FullscreenContainer, SizeFillContainer, VContainerDyn,
        },
        shapes::{Rectangle, Throbber},
        Transform,
    },
    Framework,
};
use stacks::prelude::*;

fn main() {
    Framework::run("Stacks", || {
        let mut root = VContainerDyn::new(
            ContainerSize::ZERO.expand_width().expand_height(),
            None,
        );
        root.inner_mut()
            .add_child(
                Rectangle::new(
                    LayoutSize::min(200.0, 100.0)
                        .expand_width()
                        .expand_height(),
                    Paint::new_color4f(0.2, 0.8, 0.0, 0.3),
                )
                .into(),
            )
            .add_child(
                Rectangle::new(
                    LayoutSize::min(100.0, 100.0).expand_height_by(3.0),
                    Paint::new_color4f(0.7, 0.1, 0.2, 0.3).anti_alias(),
                )
                .into(),
            )
            .add_child(
                Transform::new(
                    Rectangle::new(
                        LayoutSize::min(100.0, 50.0),
                        Paint::new_color4f(0.0, 0.0, 1.0, 0.5).anti_alias(),
                    ),
                    Matrix::scale((1.5, 1.7)),
                )
                .into(),
            )
            .add_child(
                Throbber::new(
                    LayoutDimension::min(100.0),
                    Paint::new_color4f(0.0, 1.0, 0.0, 1.0)
                        .with_stroke_width(12.0)
                        .anti_alias()
                        .stroke(),
                )
                .into(),
            )
            .add_child(
                AudioPlayer::new(
                    "resources/sound.ogg",
                    LayoutSize::min(500.0, 200.0).expand_width(),
                    Paint::new_color4f(0.8, 0.8, 1.0, 1.0).anti_alias(),
                    Paint::new_color4f(0.7, 0.7, 0.9, 0.7).anti_alias(),
                    Paint::new_color4f(1.0, 1.0, 1.0, 0.4).anti_alias(),
                )?
                .into(),
            );
        let root = SizeFillContainer::new(root, Some(Size::new(1366.0, 768.0)));
        let root = FullscreenContainer::new(root);
        let root = Audio::new(root)?;
        Ok(root)
    })
    .expect("Failed to run game");
}
