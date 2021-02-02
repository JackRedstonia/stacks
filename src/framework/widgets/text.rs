use crate::prelude::*;
use skia::{Canvas, Font as SkFont, Shaper, TextBlob};

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
    ) -> Wrap<Self> {
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
        .into()
    }

    pub fn bounds(&self) -> Size {
        self.blob
            .as_ref()
            .map(|blob| {
                let bounds = blob.bounds();
                Size::new(self.size.width, bounds.bottom + bounds.top)
            })
            .unwrap_or_default()
    }

    fn shape(&mut self) {
        let shaper = Shaper::new(None);
        self.blob = shaper
            .shape_text_blob(&self.text, &self.font, true, self.size.width, (0.0, 0.0))
            .map(|e| e.0);
    }
}

impl Widget for Text {
    fn input(&mut self, _state: &mut WidgetState, _event: &InputEvent) -> bool {
        // TODO: this is mostly a placeholder value.
        // I'm pretty sure somebody will have a use for some text to handle click events, that sort of stuff.
        false
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
        self.shape();
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        if let Some(blob) = &self.blob {
            canvas.draw_text_blob(blob, (0.0, 0.0), &self.paint);
        }
    }
}
