use super::{LayoutSize, Widget, WrapState};
use crate::game::{InputEvent, State};
use crate::skia;
use skia::{shaper::TextBlobBuilderRunHandler, Canvas, Font as SkFont, Paint, Shaper, Size};

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
    pub size: LayoutSize,
    pub text: String,
    pub font: SkFont,
    pub paint: Paint,
}

impl Text {
    pub fn new(
        size: LayoutSize,
        text: impl AsRef<str>,
        font: Font,
        style: FontStyle,
        paint: Paint,
    ) -> Self {
        let font = font.resolve(&style);
        Self {
            size,
            text: text.as_ref().to_owned(),
            paint,
            font,
        }
    }
}

impl Widget for Text {
    fn input(&mut self, _wrap: &mut WrapState, _event: &InputEvent, _size: Size) -> bool {
        // TODO: this is mostly a placeholder value.
        // I'm pretty sure somebody will have a use for some text to handle click events, that sort of stuff.
        false
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        self.size
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas, size: Size) {
        let mut handler = TextBlobBuilderRunHandler::new(&self.text, (0.0, 0.0));
        let shaper = Shaper::new(None);
        shaper.shape(&self.text, &self.font, true, size.width, &mut handler);
        if let Some(blob) = handler.make_blob() {
            let bounds = blob.bounds();
            canvas.draw_text_blob(&blob, (-bounds.left, -bounds.top), &self.paint);
        }
    }
}
