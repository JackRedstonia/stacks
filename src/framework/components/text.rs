use super::{Component, LayoutDimension, LayoutSize};
use crate::game::{Canvas, InputEvent, InputState, TimeState};
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

impl Component for Text {
    fn update(&mut self, _input_state: &InputState, _time_state: &TimeState) {}

    fn input(
        &mut self,
        _input_state: &InputState,
        _time_state: &TimeState,
        _event: &InputEvent,
        size: Size,
    ) -> bool {
        // TODO: this is mostly a placeholder value.
        // I'm pretty sure somebody will have a use for some text to handle click events, that sort of stuff.
        false
    }

    fn size(&mut self, input_state: &InputState, time_state: &TimeState) -> LayoutSize {
        // TODO: this is also mostly a placeholder value.
        // Proper text layout should be implemented - some fields in Self
        // should be added to guide text layout and that sort of stuff.
        LayoutSize::ZERO
    }

    fn draw(
        &mut self,
        _input_state: &InputState,
        _time_state: &TimeState,
        canvas: &mut Canvas,
        size: Size,
    ) {
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
