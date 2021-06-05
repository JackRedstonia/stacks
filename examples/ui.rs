use stacks::framework::{
    widgets::{
        audio::{Audio, AudioPlayer},
        layout::{
            ContainerSize, FullscreenContainer, Margin, MarginContainer,
            VContainerDyn,
        },
        shapes::Rectangle,
        ui::{Button, Slider, ValueRange},
        Backgrounded, Font, FontStyle, Fonts, Text, TextLayoutMode,
    },
    Framework,
};
use stacks::prelude::*;

fn main() {
    let e = Framework::run("Stacks", || {
        let text_paint = Paint::new_color4f(1.0, 1.0, 1.0, 1.0).anti_alias();
        let text = Text::new(
            LayoutSize::ZERO.expand_width(),
            Some(TextLayoutMode::MinHeight),
            "This demonstration features the Button and Slider widgets, along with an audio player. You can press the Space key to play/pause audio, click the blue button to seek to 25%, or change how fast the audio player's visualisations can move with the slider.",
            Font::Default,
            FontStyle::Medium,
            None,
            text_paint.clone(),
        );

        let btn_bg = Paint::new_color4f(0.2, 0.4, 0.6, 1.0).anti_alias();
        let mut button = Button::new(
            "Seek to 25%".to_owned(),
            Font::Default,
            FontStyle::Medium,
            None,
            btn_bg.clone(),
            text_paint.clone(),
        );

        let player = AudioPlayer::new(
            "resources/sound.ogg",
            LayoutSize::min(500.0, 200.0).expand_width(),
            Paint::new_color4f(0.8, 0.8, 1.0, 1.0).anti_alias(),
            Paint::new_color4f(0.7, 0.7, 0.9, 0.7).anti_alias(),
            Paint::new_color4f(1.0, 1.0, 1.0, 0.4).anti_alias(),
        )?;

        let player_weak = player.downgrade();
        let slider_btn_bg = Paint::new_color4f(0.7, 0.7, 0.9, 0.7).anti_alias();
        let mut slider = Slider::new(
            "FFT display interpolation factor".to_owned(),
            Font::Default,
            FontStyle::Medium,
            None,
            ValueRange::new(5.0..=24.0).precise_to(1.0),
            LayoutDimension::min(500.0),
            btn_bg,
            slider_btn_bg,
            text_paint,
        );
        slider.inner_mut().on_change(move |v| {
            if let Some(mut player) = player_weak.upgrade() {
                player.inner_mut().interpolation_factor = v;
            }
        });

        let player_weak = player.downgrade();
        button.inner_mut().on_click(move || {
            if let Some(mut player) = player_weak.upgrade() {
                let _ = player.inner_mut().seek_percentage(0.25);
            }
        });

        let bg_paint = Paint::new_color4f(0.1, 0.1, 0.1, 1.0).anti_alias();
        let bg = Rectangle::new(
            LayoutSize::ZERO.expand_width().expand_height(),
            bg_paint,
        );

        let mut root = VContainerDyn::new(
            ContainerSize::ZERO.expand_width().expand_height(),
            Some(18.0),
        );
        root.inner_mut()
            .add_child(text.into())
            .add_child(button.into())
            .add_child(slider.into())
            .add_child(player.into());
        let root = MarginContainer::new(root, Margin::all(18.0));
        let root = Backgrounded::new(bg, root, false);
        let root = Fonts::new(root);
        let root = FullscreenContainer::new(root);
        let root = Audio::new(root)?;
        Ok(root)
    });
    eprintln!("Failed to run game: {:?}", e);
}
