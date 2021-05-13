use stacks::framework::{
    widgets::{
        layout::{FullscreenContainer, Margin, MarginContainer},
        shapes::Rectangle,
        ui::TextEdit,
        Backgrounded, Fonts,
    },
    Framework,
};
use stacks::prelude::*;

fn main() {
    Framework::run("Stacks", || {
        let root = TextEdit::new(
            LayoutDimension::ZERO.expand(),
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
        let root = Fonts::new(root);
        let root = FullscreenContainer::new(root);
        Ok(root)
    })
    .expect("Failed to run game");
}
