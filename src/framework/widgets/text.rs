use super::{LayoutSize, Widget, WrapState};
use crate::game::{InputEvent, State};
use crate::skia;
use skia::{Paint, Size, Canvas, Font as SkFont, TextBlob};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Font {
    Default,
}

impl Font {
    pub fn resolve(&self, style: &FontStyle) -> SkFont {
        State::with(|x| x.font_set.get(self, style))
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

pub struct Text {
    pub blob: Option<TextBlob>,
    pub font: SkFont,
    pub paint: Paint,
}

impl Text {
    pub fn new(text: impl AsRef<str>, font: Font, style: FontStyle, paint: Paint) -> Self {
        let font = font.resolve(&style);
        let blob = Self::blob(text.as_ref(), &font);
        Self {
            blob, paint,
            font,
        }
    }

    fn blob(text: &str, font: &SkFont) -> Option<TextBlob> {
        TextBlob::from_str(text, font)
    }
}

impl Widget for Text {
    fn input(&mut self, _wrap: &mut WrapState, _event: &InputEvent, _size: Size) -> bool {
        // TODO: this is mostly a placeholder value.
        // I'm pretty sure somebody will have a use for some text to handle click events, that sort of stuff.
        false
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        self.blob.as_ref().map(|x| {
            let size = x.bounds().size();
            LayoutSize::min(size.width, size.height)
        }).unwrap_or(LayoutSize::ZERO)
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas, _size: Size) {
        if let Some(blob) = &self.blob {
            let bounds = blob.bounds();
            canvas.draw_text_blob(blob, (-bounds.left, -bounds.top), &self.paint);
        }
    }
}
