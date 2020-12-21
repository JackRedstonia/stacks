use super::canvas::FontSet;
use crate::framework::widgets::FontStyle;
use crate::skia;
use skia::{Font, FontStyle as SkFontStyle, Typeface};

// TODO: require argument for font set instead
pub struct DefaultFontSet {
    default_regular: Font,
    default_bold: Font,
    default_italic: Font,
    default_bold_italic: Font,
}

impl DefaultFontSet {
    pub fn new() -> Self {
        let family_name = "IBM Plex Sans";
        let size = 16.0;
        Self {
            default_regular: Font::new(
                Typeface::from_name(family_name, SkFontStyle::normal()).unwrap(),
                size,
            ),
            default_bold: Font::new(
                Typeface::from_name(family_name, SkFontStyle::bold()).unwrap(),
                size,
            ),
            default_italic: Font::new(
                Typeface::from_name(family_name, SkFontStyle::italic()).unwrap(),
                size,
            ),
            default_bold_italic: Font::new(
                Typeface::from_name(family_name, SkFontStyle::bold_italic()).unwrap(),
                size,
            ),
        }
    }
}

impl FontSet for DefaultFontSet {
    fn get_default(&self, style: FontStyle) -> &Font {
        match style {
            FontStyle::Regular => &self.default_regular,
            FontStyle::Bold => &self.default_bold,
            FontStyle::Italic => &self.default_italic,
            FontStyle::BoldItalic => &self.default_bold_italic,
        }
    }
}
