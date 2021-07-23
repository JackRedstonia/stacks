use super::super::{FontName, FontStyle, Text, TextLayoutMode};
use crate::prelude::*;

use unicode_segmentation::UnicodeSegmentation;

pub struct TextEdit {
    pub auto_focus: bool,
    pub take_input: bool,
    text: Wrap<Text>,
    text_layout_size: LayoutSize,

    size: Size,

    layout_width: LayoutDimension,

    cursor: Cursor,
    cursor_rect: Rect,
    cursor_paint: Paint,

    on_commit_fns: Vec<Box<dyn FnMut(&String)>>,
}

impl TextEdit {
    pub fn new(
        layout_size: LayoutSize,
        layout_mode: Option<TextLayoutMode>,
        font: Option<FontName>,
        style: FontStyle,
        text_size: Option<scalar>,
        text_paint: Paint,
    ) -> Wrap<Self> {
        FrameworkState::request_load();

        let text = Text::new(
            layout_size,
            layout_mode,
            "",
            font,
            style,
            text_size,
            text_paint,
        );
        Self {
            auto_focus: true,
            take_input: false,
            text,
            text_layout_size: LayoutSize::default(),
            size: Size::default(),
            layout_width: layout_size.width,
            cursor: Cursor {
                byte_offset: 0,
                position: None,
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
        self.invalidate_cursor();
    }

    fn go_left(&mut self) {
        let b = self.cursor.byte_offset;
        let text = self.text.inner_mut();
        let s = &text.get_text()[..b];
        let mut ic = false;
        if is_ctrl() {
            let p = pseudoword_start_from_end(s);
            if p != b {
                self.cursor.byte_offset = p;
                ic = true;
            }
        } else {
            let next_gr = s.grapheme_indices(true).rev().next();
            if let Some((q, _)) = next_gr {
                self.cursor.byte_offset = q;
                ic = true;
            }
        }
        drop(text);
        if ic {
            self.invalidate_cursor();
        }
    }

    fn go_right(&mut self) {
        let b = self.cursor.byte_offset;
        let text = self.text.inner_mut();
        let s = &text.get_text()[b..];
        let mut ic = false;
        if is_ctrl() {
            let p = pseudoword_start_from_start(s);
            if p != b {
                self.cursor.byte_offset = b + p;
                ic = true;
            }
        } else {
            let next_gr = s.grapheme_indices(true).next();
            ic = next_gr.is_some();
            if let Some((_, s)) = next_gr {
                self.cursor.byte_offset = b + s.len();
            }
        }
        drop(text);
        if ic {
            self.invalidate_cursor();
        }
    }

    fn backspace(&mut self) {
        let b = self.cursor.byte_offset;
        let t = self.text.inner_mut();
        let s = &t.get_text()[..b];
        if is_ctrl() {
            // Delete one Unicode word.
            let pos = pseudoword_start_from_end(s);
            drop(t);
            self.mutate_text(|tx| {
                tx.replace_range(pos..b, "");
            });
            self.cursor.byte_offset = pos;
            self.invalidate_cursor();
        } else {
            // Delete one Unicode character.
            let c = s.chars().rev().next();
            drop(t);
            if let Some(c) = c {
                self.mutate_text(|tx| {
                    tx.remove(b - c.len_utf8());
                });
                self.cursor.byte_offset -= c.len_utf8();
                self.invalidate_cursor();
            }
        }
    }

    fn delete(&mut self) {
        let b = self.cursor.byte_offset;
        let t = self.text.inner_mut();
        let s = &t.get_text()[b..];
        if is_ctrl() {
            let pos = pseudoword_start_from_start(s);
            drop(t);
            self.mutate_text(|tx| {
                tx.replace_range(b..(b + pos), "");
            })
        } else {
            let c = s.chars().next();
            drop(t);
            if c.is_some() {
                self.mutate_text(|tx| {
                    tx.remove(b);
                });
            }
        }
    }

    fn enter(&mut self) {
        if is_shift() {
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

    fn invalidate_cursor(&mut self) {
        self.cursor.position = None;
    }

    fn update_cursor(&mut self) -> Option<Vector> {
        if self.cursor.position.is_none() {
            if let Some(pos) =
                self.text.inner().grapheme_position(self.cursor.byte_offset)
            {
                self.cursor.position = Some(pos);
                return self.cursor.position;
            }
            return None;
        }
        self.cursor.position
    }
}

fn is_ctrl() -> bool {
    State::is_key_down(Keycode::LControl)
        || State::is_key_down(Keycode::RControl)
}

fn is_shift() -> bool {
    State::is_key_down(Keycode::LShift) || State::is_key_down(Keycode::RShift)
}

fn is_alt() -> bool {
    State::is_key_down(Keycode::LAlt) || State::is_key_down(Keycode::RAlt)
}

fn pseudoword_start_from_start(s: &str) -> usize {
    for (pos, st) in s.split_word_bound_indices() {
        if st.chars().any(|c| c.is_alphabetic()) {
            return pos + st.len();
        }
    }
    s.len()
}

fn pseudoword_start_from_end(s: &str) -> usize {
    for (pos, st) in s.split_word_bound_indices().rev() {
        if st.chars().any(|c| c.is_alphabetic()) {
            return pos;
        }
    }
    0
}

impl Widget for TextEdit {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.text.load(stack);
        if let Some(metrics) = self.text.inner().metrics() {
            self.cursor_rect =
                Rect::new(-0.5, metrics.ascent, 0.5, metrics.descent);
        }
        self.invalidate_cursor();
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.text.update();
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        if self.auto_focus {
            match event {
                InputEvent::MouseDown(MouseButton::Left, pos)
                    if !state.is_focused()
                        && Rect::from_size(self.size).contains(*pos) =>
                {
                    state.grab_focus();
                    self.take_input = true;
                    return true;
                }
                InputEvent::MouseDown(MouseButton::Left, pos)
                    if state.is_focused()
                        && !Rect::from_size(self.size).contains(*pos) =>
                {
                    state.release_focus();
                    self.take_input = false;
                    FrameworkState::resend_unfocused_input();
                    return false;
                }
                _ => return false,
            }
        }
        if !self.take_input {
            return false;
        }
        match event {
            InputEvent::KeyDown(k) => match k {
                Keycode::Left => {
                    self.go_left();
                }
                Keycode::Right => {
                    self.go_right();
                }
                Keycode::Back => {
                    self.backspace();
                }
                Keycode::Delete => {
                    self.delete();
                }
                Keycode::Return | Keycode::NumpadEnter => {
                    self.enter();
                }
                _ => return false,
            },
            InputEvent::CharReceived(c) if !(is_ctrl() || is_alt()) => {
                self.insert_text(&c.to_string());
            }
            _ => return false,
        }
        true
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let (mut tz, a) = self.text.size();
        tz.width = self.layout_width;
        let aa = self.text_layout_size != tz;
        self.text_layout_size = tz;

        (tz, a || aa)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
        self.text.set_size(self.text_layout_size.layout_one(size));
        self.invalidate_cursor();
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        self.text.draw(canvas);
        if self.take_input {
            let t = State::elapsed_draw().as_secs_f32();
            self.cursor_paint.set_alpha_f((t * 8.0).sin() * 0.5 + 0.5);
            if let Some(pos) = self.update_cursor() {
                canvas.draw_rect(
                    self.cursor_rect.with_offset(pos),
                    &self.cursor_paint,
                );
            }
        }
    }
}

struct Cursor {
    byte_offset: usize,
    position: Option<Vector>,
}
