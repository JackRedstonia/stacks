use super::{Widget, LayoutSize};
use crate::game::{Canvas, InputEvent};
use crate::skia;
use skia::{Paint, Size};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Font {
    Default,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

pub struct Text {
    pub text: String,
    pub font: Font,
    pub style: FontStyle,
    pub paint: Paint,
}

impl Widget for Text {
    fn input(&mut self, _event: &InputEvent, _size: Size) -> bool {
        // TODO: this is mostly a placeholder value.
        // I'm pretty sure somebody will have a use for some text to handle click events, that sort of stuff.
        false
    }

    fn size(&mut self) -> LayoutSize {
        // TODO: this is also mostly a placeholder value.
        // Proper text layout should be implemented - some fields in Self
        // should be added to guide text layout and that sort of stuff.
        LayoutSize::ZERO
    }

    fn draw(&mut self, canvas: &mut Canvas, _size: Size) {
        // TODO: text layout.
        canvas.draw_str(
            self.text.clone(),
            (0.0, 0.0),
            self.font,
            self.style,
            &self.paint,
        );
    }
}
