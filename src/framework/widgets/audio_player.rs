use crate::prelude::*;
use game::InputEvent;
use skia::{Canvas, Contains, Paint, Rect, Size};

pub struct AudioPlayer {
    pub layout_size: LayoutSize,
    pub foreground: Paint,
    pub background: Paint,
    seek_preview_percentage: Option<f32>,
    size: Size,
}

impl AudioPlayer {
    pub fn new(size: LayoutSize, foreground: Paint, background: Paint) -> Self {
        Self {
            layout_size: size,
            size: Size::new_empty(),
            foreground,
            background,
            seek_preview_percentage: None,
        }
    }
}

impl Widget for AudioPlayer {
    fn input(&mut self, wrap: &mut WrapState, event: &InputEvent) -> bool {
        match event {
            InputEvent::KeyDown(Keycode::Space) => true,
            InputEvent::MouseUp(MouseButton::Left, pos) => {
                if wrap.is_focused() {
                    wrap.release_focus();
                    self.seek_preview_percentage = None;
                }
                Rect::from_size(self.size).contains(*pos)
            }
            InputEvent::MouseDown(MouseButton::Left, pos) => {
                let c = Rect::from_size(self.size).contains(*pos);
                if c {
                    wrap.grab_focus();
                    let pos = (pos.x / self.size.width).clamp(0.0, 1.0);
                    self.seek_preview_percentage = Some(pos);
                }
                c
            }
            InputEvent::MouseMove(pos) => {
                if wrap.is_focused() || wrap.is_hovered() {
                    let pos = (pos.x / self.size.width).clamp(0.0, 1.0);
                    self.seek_preview_percentage = Some(pos);
                }
                Rect::from_size(self.size).contains(*pos)
            }
            _ => false,
        }
    }

    fn hover_lost(&mut self, _wrap: &mut WrapState) {
        self.seek_preview_percentage = None;
    }

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        canvas.draw_rect(Rect::from_size(self.size), &self.background);
        if let Some(percentage) = Some(0f32) {
            let foreground = Rect::from_wh(self.size.width * percentage, self.size.height);
            canvas.draw_rect(foreground, &self.foreground);
        }
        if let Some(preview) = self.seek_preview_percentage {
            let center = self.size.width * preview;
            let p = Rect::new(
                (center - 2.0).max(0.0),
                0.0,
                (center + 2.0).min(self.size.width),
                self.size.height,
            );
            canvas.draw_rect(p, &self.background);
        }
    }
}
