use stacks::framework::{widgets::{ui::Button, Fonts}, Framework};
use stacks::prelude::*;

fn main() {
    Framework::run("Stacks", || {
        let background = Paint::new_color4f(0.2, 0.4, 0.6, 1.0).anti_alias();
        let label = Paint::new_color4f(1.0, 1.0, 1.0, 1.0).anti_alias();
        let root = Button::new(
            "some random button, don't mind me...".to_owned(),
            background,
            label,
        );
        let root = Fonts::new().with_child(root);
        Ok(root)
    })
    .expect("Failed to run game");
}
