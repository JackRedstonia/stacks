use super::canvas::FontSet;
use crate::framework::widgets::FontStyle;
use crate::skia;
use skia::{Font, FontStyle as SkFontStyle, Typeface};

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
            default_regular: Typeface::from_name(family_name, SkFontStyle::normal()).unwrap(),
            default_bold: Typeface::from_name(family_name, SkFontStyle::bold()).unwrap(),
            default_italic: Typeface::from_name(family_name, SkFontStyle::italic()).unwrap(),
            default_bold_italic: Typeface::from_name(family_name, SkFontStyle::bold_italic())
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
