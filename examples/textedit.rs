use stacks::framework::{
    widgets::{
        layout::{FullscreenContainer, Margin, MarginContainer},
        shapes::Rectangle,
        ui::TextEdit,
        Backgrounded, FontStyle, Fonts, TextLayoutMode,
    },
    Framework,
};
use stacks::prelude::*;

fn main() {
    let e = Framework::run("Stacks", || {
        let root = TextEdit::new(
            LayoutSize::min(500.0, 0.0).expand_width().expand_height(),
            Some(TextLayoutMode::MinHeight),
            None,
            FontStyle::Regular,
            Some(48.0),
            Paint::new_color4f(1.0, 1.0, 1.0, 1.0).anti_alias(),
        );

        let bg_paint = Paint::new_color4f(0.1, 0.1, 0.1, 1.0).anti_alias();
        let bg = Rectangle::new(
            LayoutSize::ZERO.expand_width().expand_height(),
            bg_paint,
        );

        let root = MarginContainer::new(root, Margin::all(18.0));
        let root = Backgrounded::new(bg, root, false);
        let root = Fonts::new(root)?;
        let root = FullscreenContainer::new(root);
        Ok(root)
    });
    eprintln!("Failed to run game: {}", e);
}
