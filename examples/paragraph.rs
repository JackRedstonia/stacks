use stacks::framework::{
    widgets::{Font, FontStyle, Text},
    Framework,
};
use stacks::prelude::*;

fn main() {
    Framework::run("Stacks", || {
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Odio facilisis mauris sit amet massa vitae tortor condimentum lacinia. Nec dui nunc mattis enim ut tellus. Purus in mollis nunc sed id semper risus. Dolor sed viverra ipsum nunc. Amet mattis vulputate enim nulla aliquet porttitor. Ut pharetra sit amet aliquam id. Vestibulum morbi blandit cursus risus at ultrices. Maecenas accumsan lacus vel facilisis volutpat est velit egestas. Porttitor eget dolor morbi non arcu. Enim blandit volutpat maecenas volutpat blandit aliquam etiam erat velit. Id leo in vitae turpis massa sed elementum tempus. Nulla aliquet enim tortor at auctor urna nunc id cursus. Maecenas sed enim ut sem. Lectus magna fringilla urna porttitor. Nibh tortor id aliquet lectus proin nibh nisl. Interdum posuere lorem ipsum dolor sit amet consectetur. Ornare arcu odio ut sem nulla pharetra diam sit amet. Dictum at tempor commodo ullamcorper.";
        let paint = Paint::new_color4f(1.0, 1.0, 1.0, 1.0).anti_alias();
        let root = Text::new(
            LayoutSize::ZERO.expand_width().expand_height(),
            text,
            Font::Default,
            FontStyle::Regular,
            paint,
        );
        Ok(root)
    })
    .expect("Failed to run game");
}
