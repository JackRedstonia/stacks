mod fonts;

pub use fonts::{FontResource, Fonts};

use std::mem::MaybeUninit;

use crate::prelude::*;
use skia::{Canvas, Font as SkFont, FontMetrics, GlyphId, Path, RoundOut};

use unicode_linebreak::{linebreaks, BreakOpportunity};
use unicode_segmentation::UnicodeSegmentation;

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
    sk_font: Option<(Vec<SkFont>, scalar, FontMetrics)>,
    font: Font,
    style: FontStyle,
    font_size: Option<scalar>,
    size: Size,
    text: String,
    paragraph: Option<Paragraph>,
    just_changed: bool,
}

impl Text {
    pub fn new(
        size: LayoutSize,
        layout_mode: Option<TextLayoutMode>,
        text: impl AsRef<str>,
        font: Font,
        style: FontStyle,
        font_size: Option<scalar>,
        paint: Paint,
    ) -> Wrap<Self> {
        let text = text.as_ref();
        Self {
            take_input: false,
            layout_size: size,
            layout_mode: layout_mode.unwrap_or_default(),
            sk_font: None,
            font,
            style,
            font_size,
            paint,
            size: Size::new_empty(),
            text: text.to_owned(),
            paragraph: None,
            just_changed: false,
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
                } else if let Some((f, l, m)) = &self.sk_font {
                    self.paragraph = Some(Paragraph::new(
                        &self.text,
                        f,
                        *l,
                        m,
                        Some(self.size.width),
                    ));
                }
            }
            TextLayoutMode::OneLine => {
                if self.paragraph.is_none() {
                    if let Some((f, l, m)) = &self.sk_font {
                        self.paragraph =
                            Some(Paragraph::new(&self.text, f, *l, m, None));
                    }
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

    pub fn metrics(&self) -> Option<(scalar, &FontMetrics)> {
        self.sk_font.as_ref().map(|(_, l, m)| (*l, m))
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

    fn height(&self) -> scalar {
        self.paragraph
            .as_ref()
            .map(|e| e.total_height.ceil())
            .unwrap_or(0.0)
    }
}

impl Widget for Text {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some(f) = stack.get::<ResourceUser<FontResource>>() {
            if let Some(f) = f.try_access() {
                let font = f.resolve(self.font, self.style, self.font_size);
                let (line_height, metrics) = font[0].metrics();
                // Lay out text immediately if we don't care about layout sizes
                if self.layout_mode == TextLayoutMode::OneLine {
                    self.paragraph = Some(Paragraph::new(
                        &self.text,
                        &font,
                        line_height,
                        &metrics,
                        None,
                    ));
                } else {
                    // Invalidate old paragraphs
                    self.paragraph = None;
                }
                self.sk_font = Some((font, line_height, metrics));
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
    path: Path,
    bounds: Rect,
    grapheme_positions: Vec<(usize, Vector)>,
    last_position: Vector,
}

impl Word {
    fn new(s: &str, fonts: &[SkFont]) -> Self {
        assert!(!fonts.is_empty());
        let mut bounds = Rect::new_empty();
        let mut path = Path::new();
        let mut offset = Vector::default();
        let mut l = 0;
        let mut grapheme_positions = vec![];
        for chs in s.graphemes(true) {
            grapheme_positions.push((l, offset));
            let mut p = 0;
            while p < fonts.len() {
                let glyphs = fonts[p].str_to_glyphs_vec(&*chs);
                if p == fonts.len() - 1 || glyphs.iter().all(|e| *e != 0) {
                    Self::make_char(
                        &glyphs,
                        &fonts[p],
                        &mut bounds,
                        &mut offset,
                        &mut path,
                    );
                    break;
                }

                p += 1;
            }
            l += chs.len();
        }
        Self {
            string_length: s.len(),
            path,
            bounds,
            grapheme_positions,
            last_position: offset,
        }
    }

    fn make_char(
        glyphs: &[GlyphId],
        font: &SkFont,
        bounds: &mut Rect,
        offset: &mut Vector,
        path: &mut Path,
    ) {
        let mut widths = Vec::with_capacity(glyphs.len());
        unsafe {
            widths.set_len(glyphs.len());
        }
        font.get_widths(&glyphs, &mut widths);
        for (&glyph, &width) in glyphs.iter().zip(widths.iter()) {
            if let Some(glyph_path) = font.get_path(glyph) {
                let mut glb = *glyph_path.bounds();
                glb.right = glb.right.max(width);
                combine(bounds, &glb.with_offset(*offset));
                path.add_path(&glyph_path, *offset, None);
                offset.x += glb.right;
            }
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
    fn new(
        s: &str,
        font: &[SkFont],
        line_height: scalar,
        metrics: &FontMetrics,
        width: Option<scalar>,
    ) -> Self {
        assert!(!font.is_empty());
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
            // SAFETY: this is fine, we do know that `rerun_with_width` will
            // initialize this after.
            words.push((Word::new(word, font), prev_index, pb, unsafe {
                uninit_point()
            }));
        }

        if visual_prev != s.len() {
            // SAFETY: same as before, this is fine. we do know that
            // `rerun_with_width` will initialize this after.
            words.push((Word::new("", font), prev, true, unsafe {
                uninit_point()
            }))
        }

        let mut q = Self {
            words,
            bounds: Rect::new_empty(),
            line_spacing: line_height.ceil(),
            ascent: metrics.ascent.ceil(),
            descent: metrics.descent.ceil(),
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
            let word_width = word.bounds.right - word.bounds.left;
            let nx = offset.x + word_width;
            if let Some(width) = width {
                if *must_break || (nx >= width && offset.x != 0.0) {
                    offset = Vector::new(0.0, offset.y + self.line_spacing);
                    self.total_height += self.line_spacing;
                }
            }
            let b = word.bounds.with_offset(offset);
            combine(&mut self.bounds, &b);
            *word_offset = offset;
            offset.x += word_width;
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

unsafe fn uninit_point() -> Vector {
    // LINT SUPPRESSION: this is fine, really.
    // We're not getting bitten by calling deconstructors or anything,
    // Vector is just a Copy struct with 2 f32 fields.
    #[allow(clippy::uninit_assumed_init)]
    MaybeUninit::uninit().assume_init()
}
