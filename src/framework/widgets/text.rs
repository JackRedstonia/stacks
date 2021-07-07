mod fonts;

pub use fonts::{
    CachedFont, FTError, FontMetrics, FontName, FontResource, Fonts,
};

use crate::game::ID;
use crate::prelude::*;
use skia::{Canvas, RoundOut};

use unicode_linebreak::{linebreaks, BreakOpportunity};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum FontStyle {
    Regular,
    Medium,
    Bold,
    Italic,
    MediumItalic,
    BoldItalic,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum TextLayoutMode {
    /// Lays out text in the given width. Text may overflow height-wise.
    /// The layout size returned by Text will always be the same value
    /// originally given at initialization.
    Static,

    /// The layout size is ignored and all text is laid out in one line.
    OneLine,

    /// Lays out text in the given width.
    /// The height field returned by Text will be the height of the text bounds
    /// instead of the provided minimum height.
    MinHeight,
}

impl Default for TextLayoutMode {
    fn default() -> Self {
        Self::Static
    }
}

pub struct Text {
    pub take_input: bool,
    layout_size: LayoutSize,
    layout_mode: TextLayoutMode,
    paint: Paint,
    ft: ResourceUser<FontResource>,
    font_name: Option<FontName>,
    face_id: Option<ID>,
    style: FontStyle,
    font_size: Option<isize>,
    size: Size,
    text: String,
    paragraph: Option<Paragraph>,
    just_changed: bool,
    height_cache: scalar,
}

impl Text {
    pub fn new(
        size: LayoutSize,
        layout_mode: Option<TextLayoutMode>,
        text: impl AsRef<str>,
        font: Option<FontName>,
        style: FontStyle,
        font_size: Option<scalar>,
        paint: Paint,
    ) -> Wrap<Self> {
        let text = text.as_ref();
        Self {
            take_input: false,
            layout_size: size,
            layout_mode: layout_mode.unwrap_or_default(),
            ft: ResourceUser::new_none(),
            font_name: font,
            face_id: None,
            style,
            font_size: font_size.map(|e| e.to_26dot6()),
            paint,
            size: Size::new_empty(),
            text: text.to_owned(),
            paragraph: None,
            just_changed: false,
            height_cache: 0.0,
        }
        .into()
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.just_changed = true;
        self.paragraph = None;
    }

    pub fn mutate_text<F: FnMut(&mut String)>(&mut self, mut f: F) {
        f(&mut self.text);
        self.just_changed = true;
        self.paragraph = None;
    }

    pub fn force_build_paragraph(&mut self) {
        match self.layout_mode {
            TextLayoutMode::Static | TextLayoutMode::MinHeight => {
                if let Some(p) = &mut self.paragraph {
                    p.rerun_with_width(Some(self.size.width));
                } else {
                    self.make_paragraph(Some(self.size.width));
                }
            }
            TextLayoutMode::OneLine => {
                if self.paragraph.is_none() {
                    self.make_paragraph(None);
                }
            }
        }
    }

    pub fn grapheme_position(&self, byte_offset: usize) -> Option<Vector> {
        self.paragraph
            .as_ref()
            .map(|e| {
                e.grapheme_position(byte_offset)
                    .map(|e| e + self.draw_offset())
            })
            .flatten()
    }

    pub fn metrics(&self) -> Option<FontMetrics> {
        let mut ft = self.ft.try_access_mut()?;
        let f = ft.get_font(self.face_id.unwrap(), self.font_size)?;
        Some(f.metrics.clone())
    }

    pub fn draw_offset(&self) -> Vector {
        self.paragraph
            .as_ref()
            .map(|p| Vector::new(0.0, -p.ascent))
            .unwrap_or_default()
    }

    fn bounds(&self) -> Rect {
        self.paragraph
            .as_ref()
            .map(|blob| blob.bounds.round_out())
            .unwrap_or_default()
    }

    fn height(&mut self) -> scalar {
        if let Some(p) = &self.paragraph {
            self.height_cache = p.total_height.ceil();
        }
        self.height_cache
    }

    fn make_paragraph(&mut self, width: Option<scalar>) {
        let mut ft = match self.ft.try_access_mut() {
            Some(ft) => ft,
            None => return,
        };
        let f = match ft.get_font(self.face_id.unwrap(), self.font_size) {
            Some(f) => f,
            None => return,
        };
        self.paragraph = Some(Paragraph::new(&self.text, f, width));
    }
}

impl Widget for Text {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some(f) = stack.get::<ResourceUser<FontResource>>() {
            if f != &self.ft {
                self.just_changed = true;
                self.ft = f.clone();
                self.face_id = f
                    .try_access_mut()
                    .unwrap()
                    .get_face(self.font_name.as_ref(), self.style);
                self.force_build_paragraph();
            }
        }
    }

    fn update(&mut self, _state: &mut WidgetState) {}

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        self.take_input
            && event
                .position()
                .map_or(false, |p| self.bounds().contains(p))
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let just_changed = self.just_changed;
        self.just_changed = false;
        match self.layout_mode {
            TextLayoutMode::Static => (self.layout_size, just_changed),
            TextLayoutMode::OneLine => {
                let w = self.bounds().width();
                let h = self.height();
                (
                    LayoutSize::min(w, h).with_expand_from(&self.layout_size),
                    just_changed,
                )
            }
            TextLayoutMode::MinHeight => {
                let mut size = self.layout_size;
                size.height.min = self.height();
                (size, just_changed)
            }
        }
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
        self.force_build_paragraph();
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        if let Some(p) = &self.paragraph {
            canvas.save();
            canvas.translate(self.draw_offset());
            p.draw(canvas, &self.paint);
            canvas.restore();
        }
    }
}

struct Word {
    string_length: usize,
    path: skia::Path,
    advance: scalar,
    width: scalar,
    grapheme_positions: Vec<(usize, Vector)>,
    last_position: Vector,
}

impl Word {
    fn new(s: &str, fonts: &mut CachedFont) -> Self {
        let mut path = skia::Path::new();
        let mut offset = Vector::default();
        let mut width = 0.0;
        let mut l = 0;
        let mut grapheme_positions = vec![];
        for chs in s.graphemes(true) {
            grapheme_positions.push((l, offset));
            for ch in chs.chars() {
                let (glyph_path, adv) = fonts.get_char(ch);
                if !glyph_path.is_empty() {
                    width += adv;
                    path.add_path(&glyph_path, offset, None);
                }
                offset.x += adv;
            }
            l += chs.len();
        }
        Self {
            string_length: s.len(),
            path,
            advance: offset.x,
            width,
            grapheme_positions,
            last_position: offset,
        }
    }
}

struct Paragraph {
    words: Vec<(Word, usize, bool, Vector)>,
    bounds: Rect,
    line_spacing: scalar,
    ascent: scalar,
    descent: scalar,
    total_height: scalar,
}

impl Paragraph {
    fn new(s: &str, font: &mut CachedFont, width: Option<scalar>) -> Self {
        let mut prev = 0;
        let mut visual_prev = 0;
        let mut prev_break = false;

        let words_iter = linebreaks(s);
        let mut words = words_iter
            .size_hint()
            .1
            .map(|s| Vec::with_capacity(s + 1))
            .unwrap_or_else(|| Vec::new());

        for (end_index, break_op) in words_iter {
            let word = &s[prev..end_index];
            let prev_index = prev;
            prev = end_index;
            let pb = prev_break;
            prev_break = break_op == BreakOpportunity::Mandatory;
            let word = if prev_break {
                let w = word.trim_end_matches('\n');
                visual_prev = prev_index + w.len();
                w
            } else {
                visual_prev = end_index;
                word
            };
            words.push((
                Word::new(word, font),
                prev_index,
                pb,
                Vector::default(),
            ));
        }

        if visual_prev != s.len() {
            words.push((Word::new("", font), prev, true, Vector::default()))
        }

        let mut q = Self {
            words,
            bounds: Rect::new_empty(),
            line_spacing: font.metrics.height,
            ascent: font.metrics.ascent,
            descent: font.metrics.descent,
            total_height: 0.0,
        };
        q.rerun_with_width(width);
        q
    }

    fn rerun_with_width(&mut self, width: Option<scalar>) {
        self.bounds = Rect::new_empty();
        let mut offset = Vector::default();
        self.total_height = -self.ascent + self.descent;
        for (word, _, must_break, word_offset) in &mut self.words {
            let nx = offset.x + word.width;
            if let Some(width) = width {
                if *must_break || (nx >= width && offset.x != 0.0) {
                    offset = Vector::new(0.0, offset.y + self.line_spacing);
                    self.total_height += self.line_spacing;
                }
            }
            let b = Rect::new(0.0, self.ascent, word.width, self.descent)
                .with_offset(offset);
            combine(&mut self.bounds, &b);
            *word_offset = offset;
            offset.x += word.advance;
        }
    }

    fn draw(&self, canvas: &mut Canvas, paint: &Paint) {
        for (word, _, _, position) in &self.words {
            canvas.save();
            canvas.translate(*position);
            canvas.draw_path(&word.path, &paint);
            canvas.restore();
        }
    }

    fn grapheme_position(&self, pos: usize) -> Option<Vector> {
        if self.words.is_empty() && pos == 0 {
            return Some(Vector::default());
        }
        for (word, byte_offset, _, word_offset) in &self.words {
            let to = *byte_offset + word.string_length;
            if pos == to {
                return Some(word.last_position + *word_offset);
            }
            if (*byte_offset..to).contains(&pos) {
                let pos = pos - byte_offset;
                let mut prev_offset = Vector::default();
                for &(b, p) in &word.grapheme_positions {
                    if pos == b {
                        return Some(p + *word_offset);
                    }
                    if pos < b {
                        return Some(prev_offset + *word_offset);
                    }
                    prev_offset = p;
                }
                if let Some(pos) =
                    word.grapheme_positions.iter().position(|(b, _)| pos < *b)
                {
                    return word
                        .grapheme_positions
                        .get(pos - 1)
                        .map(|(_, p)| *p + *word_offset);
                }
            }
        }
        None
    }
}

// TODO: promote this to an utils function
fn combine(a: &mut Rect, b: &Rect) {
    a.left = a.left.min(b.left);
    a.right = a.right.max(b.right);
    a.top = a.top.min(b.top);
    a.bottom = a.bottom.max(b.bottom);
}
