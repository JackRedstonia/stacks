use super::super::{Font, FontStyle, Text, TextLayoutMode};
use crate::prelude::*;

use unicode_segmentation::UnicodeSegmentation;

pub struct TextEdit {
    text: Wrap<Text>,
    text_size: LayoutSize,

    size: Size,

    layout_width: LayoutDimension,

    cursor: Cursor,
    cursor_rect: Rect,
    cursor_paint: Paint,

    on_commit_fns: Vec<Box<dyn FnMut(&String)>>,
}

impl TextEdit {
    pub fn new(
        layout_width: LayoutDimension,
        text_size: Option<scalar>,
        text_paint: Paint,
    ) -> Wrap<Self> {
        let text = Text::new(
            LayoutSize::ZERO.expand_width(),
            Some(TextLayoutMode::MinHeight),
            "",
            Font::Default,
            FontStyle::Regular,
            text_size,
            text_paint,
        );
        Self {
            text,
            text_size: LayoutSize::default(),
            size: Size::default(),
            layout_width,
            cursor: Cursor {
                byte_offset: 0,
                position: Vector::default(),
            },
            cursor_rect: Rect::default(),
            cursor_paint: Paint::new_color4f(1.0, 1.0, 1.0, 1.0).anti_alias(),
            on_commit_fns: vec![],
        }
        .into()
    }

    pub fn on_commit<F: 'static + for<'r> FnMut(&'r String)>(&mut self, f: F) {
        self.on_commit_fns.push(Box::new(f));
    }

    fn insert_text(&mut self, s: &str) {
        let b = self.cursor.byte_offset;
        self.mutate_text(|tx| {
            tx.insert_str(b, s);
        });
        self.cursor.byte_offset += s.len();
    }

    fn go_left(&mut self) {
        let b = self.cursor.byte_offset;
        let text = self.text.inner_mut();
        let s = &text.get_text()[..b];
        let next_gr = s.grapheme_indices(true).rev().next();
        if let Some((q, _)) = next_gr {
            self.cursor.byte_offset = q;
        }
        drop(text);
        self.update_cursors();
    }

    fn go_right(&mut self) {
        let b = self.cursor.byte_offset;
        let text = self.text.inner_mut();
        let s = &text.get_text()[b..];
        let next_gr = s.grapheme_indices(true).next();
        if let Some((_, s)) = next_gr {
            self.cursor.byte_offset = b + s.len();
        }
        drop(text);
        self.update_cursors();
    }

    fn backspace(&mut self) {
        let b = self.cursor.byte_offset;
        let t = self.text.inner_mut();
        let c = t.get_text()[..b].chars().rev().next();
        drop(t);
        if let Some(c) = c {
            self.mutate_text(|tx| {
                tx.remove(b - c.len_utf8());
            });
            self.cursor.byte_offset -= c.len_utf8();
        }
    }

    fn delete(&mut self) {
        let b = self.cursor.byte_offset;
        let t = self.text.inner_mut();
        let c = t.get_text()[b..].chars().next();
        drop(t);
        if c.is_some() {
            self.mutate_text(|tx| {
                tx.remove(b);
            });
        }
    }

    fn enter(&mut self) {
        let is_shift = State::is_key_down(Keycode::LShift)
            || State::is_key_down(Keycode::RShift);
        if is_shift {
            self.insert_text("\n");
        } else {
            let text = self.text.inner_mut();
            let text = text.get_text();
            for i in &mut self.on_commit_fns {
                i(text);
            }
        }
    }

    fn mutate_text<F: for<'r> FnMut(&'r mut String)>(&mut self, f: F) {
        let mut t = self.text.inner_mut();
        t.mutate_text(f);
        t.force_build_paragraph();
    }

    fn update_cursors(&mut self) {
        if let Some(pos) =
            self.text.inner().grapheme_position(self.cursor.byte_offset)
        {
            self.cursor.position = pos;
        }
    }
}

impl Widget for TextEdit {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.text.load(stack);
        if let Some((_, metrics)) = self.text.inner().metrics() {
            self.cursor_rect =
                Rect::new(-0.5, metrics.ascent, 0.5, metrics.descent);
        }
        self.update_cursors();
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.text.update();
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        if !state.is_focused() {
            match event {
                InputEvent::MouseDown(MouseButton::Left, pos)
                    if Rect::from_size(self.size).contains(*pos) =>
                {
                    state.grab_focus();
                    return true;
                }
                _ => return false,
            }
        }
        match event {
            InputEvent::MouseDown(MouseButton::Left, pos)
                if !Rect::from_size(self.size).contains(*pos) =>
            {
                state.release_focus();
            }
            InputEvent::KeyDown(k) => match k {
                Keycode::Left => {
                    self.go_left();
                }
                Keycode::Right => {
                    self.go_right();
                }
                Keycode::Backspace => {
                    self.backspace();
                }
                Keycode::Delete => {
                    self.delete();
                }
                Keycode::Return | Keycode::KpEnter => {
                    self.enter();
                }
                _ => return false,
            },
            InputEvent::TextInput(s) => {
                self.insert_text(s);
            }
            _ => return false,
        }
        true
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let (mut tz, a) = self.text.size();
        tz.width = self.layout_width;
        let aa = self.text_size != tz;
        self.text_size = tz;

        (tz, a || aa)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
        self.text.set_size(self.text_size.layout_one(size));
        self.update_cursors();
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        self.text.draw(canvas);
        if state.is_focused() {
            let t = State::elapsed_draw().as_secs_f32();
            self.cursor_paint.set_alpha_f((t * 8.0).sin() * 0.5 + 0.5);
            canvas.draw_rect(
                self.cursor_rect.with_offset(self.cursor.position),
                &self.cursor_paint,
            );
        }
    }
}

struct Cursor {
    byte_offset: usize,
    position: Vector,
}
