use crate::framework::widgets::{Font, FontStyle};
use crate::skia::Font as SkFont;

pub trait FontSet {
    fn get(&self, font: &Font, style: &FontStyle) -> SkFont {
        match font {
            Font::Default => self.get_default(style),
        }
    }

    fn get_default(&self, style: &FontStyle) -> SkFont;
}
