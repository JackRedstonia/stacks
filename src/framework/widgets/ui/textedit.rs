use super::super::{Font, FontStyle, Text, TextLayoutMode};
use crate::prelude::*;

use unicode_segmentation::UnicodeSegmentation;

pub struct TextEdit {
    font: Font,
    font_style: FontStyle,
    text_size: Option<f32>,
    text_paint: Paint,

    text: Wrap<Text>,
    text_layout_size: LayoutSize,
    text_editing: Option<Wrap<Text>>,
    text_editing_layout_size: LayoutSize,

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
        layout_mode: Option<TextLayoutMode>,
        text_size: Option<scalar>,
        text_paint: Paint,
    ) -> Wrap<Self> {
        let font = Font::Default;
        let font_style = FontStyle::Regular;
        let text = Text::new(
            LayoutSize::ZERO.expand_width(),
            layout_mode,
            "",
            font,
            font_style,
            text_size,
            text_paint.clone(),
        );
        Self {
            font,
            font_style,
            text_size,
            text_paint,

            text,
            text_layout_size: LayoutSize::default(),
            text_editing: None,
            text_editing_layout_size: LayoutSize::default(),
            size: Size::default(),
            layout_width,
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
        return self.cursor.position;
    }
}

fn is_ctrl() -> bool {
    State::is_key_down(Keycode::LCtrl) || State::is_key_down(Keycode::RCtrl)
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
    return s.len();
}

fn pseudoword_start_from_end(s: &str) -> usize {
    for (pos, st) in s.split_word_bound_indices().rev() {
        if st.chars().any(|c| c.is_alphabetic()) {
            return pos;
        }
    }
    return 0;
}

impl Widget for TextEdit {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some(text_editing) = self.text_editing.as_mut() {
            text_editing.load(stack);
        }
        self.text.load(stack);
        if let Some((_, metrics)) = self.text.inner().metrics() {
            self.cursor_rect =
                Rect::new(-0.5, metrics.ascent, 0.5, metrics.descent);
        }
        self.invalidate_cursor();
    }

    fn update(&mut self, _state: &mut WidgetState) {
        if let Some(text_editing) = self.text_editing.as_mut() {
            text_editing.update();
        }
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
            InputEvent::TextEditing(s) if !(is_ctrl() || is_alt()) => {
                if let Some(text_editing) = self.text_editing.as_mut() {
                    text_editing.inner_mut().mutate_text(|tx| {
                        *tx = s.clone();
                    });
                } else {
                    self.text_editing = Some(Text::new(
                        LayoutSize::ZERO.expand_width(),
                        Some(TextLayoutMode::OneLine),
                        s,
                        self.font,
                        self.font_style,
                        self.text_size,
                        self.text_paint
                            .clone()
                            .with_alpha(self.text_paint.alpha_f() * 0.3),
                    ));
                    FrameworkState::request_load();
                }
            }
            InputEvent::TextInput(s) if !(is_ctrl() || is_alt()) => {
                self.text_editing = None;
                self.insert_text(s);
            }
            _ => return false,
        }
        true
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let mut ea = false;
        if let Some(text_editing) = self.text_editing.as_mut() {
            let (s, a) = text_editing.size();
            self.text_editing_layout_size = s;
            ea = a;
        }

        let (mut tz, a) = self.text.size();
        tz.width = self.layout_width;
        let aa = self.text_layout_size != tz;
        self.text_layout_size = tz;

        (tz, a || aa || ea)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        if let Some(text_editing) = self.text_editing.as_mut() {
            text_editing.set_size(
                self.text_editing_layout_size.layout_one(Size::default()),
            );
        }

        self.size = size;
        self.text.set_size(self.text_layout_size.layout_one(size));
        self.invalidate_cursor();
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        self.text.draw(canvas);
        if state.is_focused() {
            let t = State::elapsed_draw().as_secs_f32();
            self.cursor_paint.set_alpha_f((t * 8.0).sin() * 0.5 + 0.5);
            if let Some(pos) = self.update_cursor() {
                canvas.draw_rect(
                    self.cursor_rect.with_offset(pos),
                    &self.cursor_paint,
                );
            }
        }
        if let Some(text_editing) = self.text_editing.as_mut() {
            canvas.save();
            if let Some(pos) = self.cursor.position {
                canvas.translate(pos - text_editing.inner().draw_offset());
            }
            text_editing.draw(canvas);
            canvas.restore();
        }
    }
}

struct Cursor {
    byte_offset: usize,
    position: Option<Vector>,
}
