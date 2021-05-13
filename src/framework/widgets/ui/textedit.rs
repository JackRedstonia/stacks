use super::super::{
    layout::{Margin, MarginContainer},
    Font, FontStyle, Text, TextLayoutMode,
};
use crate::prelude::*;

use unicode_segmentation::UnicodeSegmentation;

pub struct TextEdit {
    text: Wrap<MarginContainer<TextCursors>>,
    text_inner: Wrap<TextCursors>,
    text_size: LayoutSize,

    size: Size,

    layout_width: LayoutDimension,
}

impl TextEdit {
    pub fn new(
        layout_width: LayoutDimension,
        text_size: Option<scalar>,
        text_paint: Paint,
    ) -> Wrap<Self> {
        let text = TextCursors::new(text_size, text_paint);
        let text_inner = text.clone();
        let text = MarginContainer::new(text, Margin::vertical(10.0));
        Self {
            text,
            text_inner,
            text_size: LayoutSize::default(),
            size: Size::default(),
            layout_width,
        }
        .into()
    }
}

impl Widget for TextEdit {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.text.load(stack);
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.text.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        match event {
            InputEvent::KeyDown(Keycode::Left) => {
                let mut cursor = self.text_inner.inner_mut();
                let mut chk = || {
                    let b = cursor.cursor.byte_offset;
                    let text = cursor.text.inner_mut();
                    let text = &text.get_text()[..b];
                    let next_gr = text.grapheme_indices(true).rev().next();
                    if let Some((q, _)) = next_gr {
                        return q;
                    }
                    return b;
                };
                cursor.cursor.byte_offset = chk();
                cursor.update_cursors();
                return true;
            }
            InputEvent::KeyDown(Keycode::Right) => {
                let mut cursor = self.text_inner.inner_mut();
                let mut chk = || {
                    let b = cursor.cursor.byte_offset;
                    let text = cursor.text.inner_mut();
                    let text = &text.get_text()[b..];
                    let next_gr = text.grapheme_indices(true).next();
                    if let Some((_, s)) = next_gr {
                        return b + s.len();
                    }
                    return b;
                };
                cursor.cursor.byte_offset = chk();
                cursor.update_cursors();
                return true;
            }
            InputEvent::KeyDown(k) => {
                let mut text = self.text_inner.inner_mut();
                let b = text.cursor.byte_offset;
                if k == &Keycode::Backspace {
                    let t = text.text.inner_mut();
                    let c = t.get_text()[..b].chars().rev().next();
                    drop(t);
                    if let Some(c) = c {
                        text.text.inner_mut().mutate_text(|tx| {
                            tx.remove(b - c.len_utf8());
                        });
                        text.cursor.byte_offset -= c.len_utf8();
                    }
                    return true;
                } else if k == &Keycode::Delete {
                    let t = text.text.inner_mut();
                    let c = t.get_text()[b..].chars().next();
                    drop(t);
                    if c.is_some() {
                        text.text.inner_mut().mutate_text(|tx| {
                            tx.remove(b);
                        });
                    }
                }
            }
            InputEvent::TextInput(s) => {
                let mut text = self.text_inner.inner_mut();
                let b = text.cursor.byte_offset;
                text.text.inner_mut().mutate_text(|tx| {
                    tx.insert_str(b, &s);
                });
                text.cursor.byte_offset += s.len();
                return true;
            }
            _ => {}
        }
        return false;
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
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        self.text.draw(canvas);
    }
}

struct Cursor {
    byte_offset: usize,
    position: Vector,
}

struct TextCursors {
    text: Wrap<Text>,
    cursor: Cursor,
}

impl TextCursors {
    fn new(text_size: Option<scalar>, text_paint: Paint) -> Wrap<Self> {
        let text = Text::new(
            LayoutSize::ZERO.expand_width(),
            Some(TextLayoutMode::MinHeight),
            "some example text for you to cursor around, haha",
            Font::Default,
            FontStyle::Regular,
            text_size,
            text_paint,
        );
        Self {
            text,
            cursor: Cursor {
                byte_offset: 0,
                position: Vector::default(),
            },
        }
        .into()
    }

    fn update_cursors(&mut self) {
        if let Some(pos) =
            self.text.inner().grapheme_position(self.cursor.byte_offset)
        {
            self.cursor.position = pos;
        }
    }
}

impl Widget for TextCursors {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.text.load(stack);
        self.update_cursors();
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.text.update();
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        self.text.size()
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.text.set_size(size);
        self.update_cursors();
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        self.text.draw(canvas);
        if let Some((_, metrics)) = self.text.inner_mut().metrics() {
            canvas.draw_rect(
                Rect::new(-0.5, metrics.ascent, 0.5, metrics.descent)
                    .with_offset(self.cursor.position),
                &Paint::new_color4f(1.0, 1.0, 1.0, (State::elapsed_draw().as_secs_f32() * 6.0).sin() * 0.5 + 0.5).anti_alias(),
            );
        }
    }
}
