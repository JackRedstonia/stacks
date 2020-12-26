use super::{LayoutSize, Widget, WrapState};
use crate::game::{InputEvent, State};
use crate::skia;
use skia::{Paint, Size, Canvas, Font as SkFont};

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
    pub text: String,
    pub font: Font,
    pub style: FontStyle,
    pub paint: Paint,
}

impl Widget for Text {
    fn input(&mut self, _wrap: &mut WrapState, _event: &InputEvent, _size: Size) -> bool {
        // TODO: this is mostly a placeholder value.
        // I'm pretty sure somebody will have a use for some text to handle click events, that sort of stuff.
        false
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        // TODO: this is also mostly a placeholder value.
        // Proper text layout should be implemented - some fields in Self
        // should be added to guide text layout and that sort of stuff.
        LayoutSize::ZERO
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas, _size: Size) {
        // TODO: text layout.
        canvas.draw_str(
            &self.text,
            (0.0, 0.0),
            &self.font.resolve(&self.style),
            &self.paint,
        );
    }
}
