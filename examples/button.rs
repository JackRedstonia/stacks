use stacks::framework::{
    widgets::{
        audio::{Audio, AudioPlayer},
        layout::{ContainerSize, VContainer},
        ui::Button,
        Fonts,
    },
    Framework,
};
use stacks::prelude::*;

fn main() {
    Framework::run("Stacks", || {
        let background = Paint::new_color4f(0.2, 0.4, 0.6, 1.0).anti_alias();
        let label = Paint::new_color4f(1.0, 1.0, 1.0, 1.0).anti_alias();
        let mut button = Button::new(
            "click to seek music player to 25%".to_owned(),
            Some(24.0),
            background,
            label,
        );
        let player = AudioPlayer::new(
            "resources/sound.ogg",
            LayoutSize::min(500.0, 200.0).expand_width(),
            Paint::new_color4f(0.8, 0.8, 1.0, 1.0).anti_alias(),
            Paint::new_color4f(0.7, 0.7, 0.9, 0.7).anti_alias(),
            Paint::new_color4f(1.0, 1.0, 1.0, 0.4).anti_alias(),
        )?;
        let player_weak = player.downgrade();
        button.inner().on_click(move || {
            if let Some(mut player) = player_weak.upgrade() {
                let _ = player.inner().seek_percentage(0.25);
            }
        });
        let root =
            VContainer::new(ContainerSize::ZERO.expand_width().expand_height())
                .with_child(button)
                .with_child(player);
        let root = Fonts::new().with_child(root);
        let root = Audio::new()?.with_child(root);
        Ok(root)
    })
    .expect("Failed to run game");
}
