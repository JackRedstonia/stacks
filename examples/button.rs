use stacks::framework::{
    widgets::{
        audio::{Audio, AudioPlayer},
        layout::{ContainerSize, Margin, MarginContainer, VContainer},
        ui::Button,
        Font, FontStyle, Fonts, Text, TextLayoutMode,
    },
    Framework,
};
use stacks::prelude::*;

fn main() {
    Framework::run("Stacks", || {
        let text_paint = Paint::new_color4f(1.0, 1.0, 1.0, 1.0).anti_alias();
        let text = Text::new(
            LayoutSize::ZERO.expand_width(),
            Some(TextLayoutMode::MinHeight),
            "This demo features Button along with AudioPlayer. Press the Space key to play/pause music, or click the blue button below to seek to 25%.",
            Font::Default,
            FontStyle::Regular,
            None,
            text_paint.clone(),
        );
        let text = MarginContainer::new(Margin::all(24.0)).with_child(text);

        let background = Paint::new_color4f(0.2, 0.4, 0.6, 1.0).anti_alias();
        let mut button = Button::new(
            "click to seek music player to 25%".to_owned(),
            None,
            background,
            text_paint,
        );

        let player = AudioPlayer::new(
            "resources/sound.ogg",
            LayoutSize::min(500.0, 200.0).expand_width(),
            Paint::new_color4f(0.8, 0.8, 1.0, 1.0).anti_alias(),
            Paint::new_color4f(0.7, 0.7, 0.9, 0.7).anti_alias(),
            Paint::new_color4f(1.0, 1.0, 1.0, 0.4).anti_alias(),
        )?;

        let player_weak = player.downgrade();
        button.inner_mut().on_click(move || {
            if let Some(mut player) = player_weak.upgrade() {
                let _ = player.inner_mut().seek_percentage(0.25);
            }
        });

        let root =
            VContainer::new(ContainerSize::ZERO.expand_width().expand_height(), None)
                .with_child(text)
                .with_child(button)
                .with_child(player);
        let root = Fonts::new().with_child(root);
        let root = Audio::new()?.with_child(root);
        Ok(root)
    })
    .expect("Failed to run game");
}
