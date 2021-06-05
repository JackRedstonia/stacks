use stacks::framework::{
    widgets::{
        layout::{
            ContainerSize, FullscreenContainer, Margin, MarginContainer,
            ScrollContainer, VContainerDyn,
        },
        shapes::Rectangle,
        Backgrounded, Font, FontStyle, Fonts, Text, TextLayoutMode,
    },
    Framework,
};
use stacks::prelude::*;

fn main() {
    let e = Framework::run("Stacks", || {
        let paint = Paint::new_color4f(1.0, 1.0, 1.0, 1.0).anti_alias();

        let header = "The chunk of text below can be scrolled through.\nTry scrolling with your mouse and resizing the window to see how the text automatically lays itself out to fit the window's width.";
        let header = Text::new(
            LayoutSize::ZERO.expand_width(),
            Some(TextLayoutMode::MinHeight),
            header,
            Font::Default,
            FontStyle::Medium,
            None,
            paint.clone(),
        );

        let text = "環トソホ報質ぜぶゆて開場決下ト教平スち託在7加ヲ市置げせわ呼同監毎スそら段情ヱヒケス年問尊限且佃ぜあ。載トづルゅ需像ヱリハル並転イコソト納長マハ法不フ筋給ム日省ヌ療福ゃひてし問奉ぱ要2施録ぼずぎす大料解づゆ。事ネ産亡ヤナユ読領ぴちルよ国再どドょ写断にくとゆ真検びレた個物リ大揮ンまに子認高ナヒイ別96乾凍刷往4財トオモ著気いねべ欠広やぞこぱ説梨ヨ提体ラるだ窃術座め存常連夫補おッ。".repeat(6);
        let text = Text::new(
            LayoutSize::ZERO.expand_width(),
            Some(TextLayoutMode::MinHeight),
            text,
            Font::Default,
            FontStyle::Regular,
            Some(24.0),
            paint,
        );
        let text = ScrollContainer::new(
            text,
            LayoutSize::ZERO.expand_width().expand_height(),
        );

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
            .add_child(header.into())
            .add_child(text.into());
        let root = MarginContainer::new(root, Margin::all(18.0));
        let root = Backgrounded::new(bg, root, false);
        let root = Fonts::new(root);
        let root = FullscreenContainer::new(root);
        Ok(root)
    });
    eprintln!("Failed to run game: {:?}", e);
}
