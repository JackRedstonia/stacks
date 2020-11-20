use super::Component;
use crate::game::{
    input::{InputEvent, InputState},
    time::TimeState,
};
use crate::canvas::Canvas;
use skia_safe::Paint;
use skulpin_renderer::skia_safe;

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

    fn draw(&mut self, _input_state: &InputState, _time_state: &TimeState, canvas: &mut Canvas) {
        canvas.draw_str(
            self.text.clone(),
            (0.0, 0.0),
            self.font,
            self.style,
            &self.paint,
        );
    }

    fn input(&mut self, _input_state: &InputState, _time_state: &TimeState, _event: &InputEvent) {}
}
