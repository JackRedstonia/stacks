use crate::prelude::*;
use game::{InputEvent, State};
use skia::{
    shaper::TextBlobBuilderRunHandler, Canvas, Font as SkFont, Paint, Shaper, Size, TextBlob,
};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Font {
    Default,
}

impl Font {
    pub fn resolve(&self, style: &FontStyle) -> SkFont {
        State::get_font_set(self, style)
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
    pub layout_size: LayoutSize,
    pub font: SkFont,
    pub paint: Paint,
    size: Size,
    text: String,
    blob: Option<TextBlob>,
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
        let text = text.as_ref();
        Self {
            layout_size: size,
            font,
            paint,
            size: Size::new_empty(),
            text: text.to_owned(),
            blob: None,
        }
    }

    fn shape(&mut self) {
        let mut handler = TextBlobBuilderRunHandler::new(&self.text, (0.0, 0.0));
        let shaper = Shaper::new(None);
        shaper.shape(&self.text, &self.font, true, self.size.width, &mut handler);
        self.blob = handler.make_blob();
    }
}

impl Widget for Text {
    fn input(&mut self, _wrap: &mut WidgetState, _event: &InputEvent) -> bool {
        // TODO: this is mostly a placeholder value.
        // I'm pretty sure somebody will have a use for some text to handle click events, that sort of stuff.
        false
    }

    fn size(&mut self, _wrap: &mut WidgetState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _wrap: &mut WidgetState, size: Size) {
        self.size = size;
        self.shape();
    }

    fn draw(&mut self, _wrap: &mut WidgetState, canvas: &mut Canvas) {
        if let Some(blob) = &self.blob {
            let bounds = blob.bounds();
            canvas.draw_text_blob(blob, (-bounds.left, -bounds.top), &self.paint);
        }
    }
}
