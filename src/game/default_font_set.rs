use super::canvas::FontSet;
use crate::framework::widgets::FontStyle;
use crate::skia;
use skia::{
    font_style::{Slant, Weight, Width},
    Font, FontStyle as SkFontStyle, Typeface,
};

// TODO: require argument for font set instead
pub struct DefaultFontSet {
    size: f32,
    default_regular: Typeface,
    default_bold: Typeface,
    default_italic: Typeface,
    default_bold_italic: Typeface,
}

impl DefaultFontSet {
    pub fn new() -> Self {
        let family_name = "IBM Plex Sans";
        Self {
            size: 16.0,
            default_regular: Typeface::from_name(
                family_name,
                SkFontStyle::new(Weight::MEDIUM, Width::NORMAL, Slant::Upright),
            )
            .unwrap(),
            default_bold: Typeface::from_name(
                family_name,
                SkFontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Upright),
            )
            .unwrap(),
            default_italic: Typeface::from_name(
                family_name,
                SkFontStyle::new(Weight::MEDIUM, Width::NORMAL, Slant::Italic),
            )
            .unwrap(),
            default_bold_italic: Typeface::from_name(
                family_name,
                SkFontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Italic),
            )
            .unwrap(),
        }
    }
}

impl FontSet for DefaultFontSet {
    fn get_default(&self, style: &FontStyle) -> Font {
        Font::new(
            match style {
                FontStyle::Regular => &self.default_regular,
                FontStyle::Bold => &self.default_bold,
                FontStyle::Italic => &self.default_italic,
                FontStyle::BoldItalic => &self.default_bold_italic,
            },
            self.size,
        )
    }
}
