use stacks::framework::{
    widgets::{Font, FontStyle, Fonts, Text},
    Framework,
};
use stacks::prelude::*;

fn main() {
    Framework::run("Stacks", || {
        let text = "hello world i am text 環トソホ報質ぜぶゆて開場決下ト教平スち託在7加ヲ市置げせわ呼同監毎スそら段情ヱヒケス年問尊限且佃ぜあ。載トづルゅ需像ヱリハル並転イコソト納長マハ法不フ筋給ム日省ヌ療福ゃひてし問奉ぱ要2施録ぼずぎす大料解づゆ。事ネ産亡ヤナユ読領ぴちルよ国再どドょ写断にくとゆ真検びレた個物リ大揮ンまに子認高ナヒイ別96乾凍刷往4財トオモ著気いねべ欠広やぞこぱ説梨ヨ提体ラるだ窃術座め存常連夫補おッ。";
        let paint = Paint::new_color4f(1.0, 1.0, 1.0, 1.0).anti_alias();
        let root = Text::new(
            LayoutSize::ZERO.expand_width().expand_height(),
            None,
            text,
            Font::Default,
            FontStyle::Regular,
            Some(24.0),
            paint,
        );
        let root = Fonts::new().with_child(root);
        Ok(root)
    })
    .expect("Failed to run game");
}
