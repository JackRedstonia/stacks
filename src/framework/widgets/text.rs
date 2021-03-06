mod fonts;

pub use fonts::{FontResource, Fonts};

use std::mem::MaybeUninit;

use crate::prelude::*;
use skia::{Canvas, Font as SkFont, GlyphId, Path};

use unicode_linebreak::{linebreaks, BreakOpportunity};

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
    sk_font: Option<Vec<SkFont>>,
    font: Font,
    style: FontStyle,
    font_size: Option<scalar>,
    size: Size,
    text: String,
    paragraph: Option<Paragraph>,
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
        }
        .into()
    }

    pub fn bounds(&self) -> Rect {
        self.paragraph
            .as_ref()
            .map(|blob| blob.bounds)
            .unwrap_or_default()
    }
}

impl Widget for Text {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some(f) = stack.get::<ResourceUser<FontResource>>() {
            if let Some(f) = f.try_access() {
                self.sk_font =
                    Some(f.resolve(self.font, self.style, self.font_size));
                // Lay out text immediately if we don't care about layout sizes
                if self.layout_mode == TextLayoutMode::OneLine {
                    self.paragraph = Some(Paragraph::new(
                        &self.text,
                        self.sk_font.as_ref().unwrap(),
                        None,
                    ));
                } else {
                    // Invalidate old paragraphs
                    self.paragraph = None;
                }
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
        match self.layout_mode {
            TextLayoutMode::Static => (self.layout_size, false),
            TextLayoutMode::OneLine => {
                let b = self.bounds();
                let w = b.width();
                let h = b.height();
                (
                    LayoutSize::min(w, h).with_expand_from(&self.layout_size),
                    false,
                )
            }
            TextLayoutMode::MinHeight => {
                let b = self.bounds();
                let h = b.height();
                let mut size = self.layout_size;
                size.height.min = h;
                (size, false)
            }
        }
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
        match self.layout_mode {
            TextLayoutMode::Static | TextLayoutMode::MinHeight => {
                if let Some(p) = &mut self.paragraph {
                    p.rerun_with_width(Some(size.width));
                } else if let Some(f) = &self.sk_font {
                    self.paragraph =
                        Some(Paragraph::new(&self.text, f, Some(size.width)));
                }
            }
            TextLayoutMode::OneLine => {
                if self.paragraph.is_none() {
                    if let Some(f) = &self.sk_font {
                        self.paragraph =
                            Some(Paragraph::new(&self.text, f, None));
                    }
                }
            }
        }
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        if let Some(p) = &self.paragraph {
            p.draw(canvas, &self.paint);
        }
    }
}

struct Word {
    path: Path,
    bounds: Rect,
}

impl Word {
    fn new(s: &str, font: &[SkFont]) -> Self {
        assert!(!font.is_empty());
        let mut bounds = Rect::new_empty();
        let mut path = Path::new();
        let mut offset = Vector::default();
        use unicode_segmentation::UnicodeSegmentation;
        for chs in s.graphemes(true) {
            let mut p = 0;
            while p < font.len() {
                let glyphs = font[p].str_to_glyphs_vec(&*chs);
                if p == font.len() - 1 || glyphs.iter().all(|e| *e != 0) {
                    Self::make_char(
                        &glyphs,
                        &font[p],
                        &mut bounds,
                        &mut offset,
                        &mut path,
                    );
                    break;
                }

                p += 1;
            }
        }
        Self { path, bounds }
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
                let mut glb = glyph_path.bounds().clone();
                glb.right = glb.right.max(width);
                combine(bounds, &glb.with_offset(*offset));
                path.add_path(&glyph_path, *offset, None);
                offset.x += glb.right;
            }
        }
    }
}

struct Paragraph {
    words: Vec<(Word, bool, Vector)>,
    bounds: Rect,
    line_height: scalar,
}

impl Paragraph {
    fn new(s: &str, font: &[SkFont], width: Option<scalar>) -> Self {
        assert!(!font.is_empty());
        let mut prev = 0;
        let mut prev_break = false;
        let words = linebreaks(s)
            .map(|(end_index, break_op)| {
                let word = &s[prev..end_index];
                prev = end_index;
                let pb = prev_break;
                prev_break = matches!(break_op, BreakOpportunity::Mandatory);
                let word = if prev_break {
                    word.trim_end_matches('\n')
                } else {
                    word
                };
                (Word::new(word, font), pb, unsafe {
                    MaybeUninit::uninit().assume_init()
                })
            })
            .collect();
        let mut s = Self {
            words,
            bounds: Rect::new_empty(),
            line_height: font[0].metrics().0,
        };
        s.rerun_with_width(width);
        s
    }

    fn rerun_with_width(&mut self, width: Option<scalar>) {
        self.bounds = Rect::new_empty();
        let mut offset = Vector::default();
        for (word, must_break, word_offset) in &mut self.words {
            let nx = offset.x + word.bounds.right - word.bounds.left;
            if let Some(width) = width {
                if *must_break || (nx >= width && offset.x != 0.0) {
                    offset = Vector::new(0.0, offset.y + self.line_height);
                }
            }
            let b = word.bounds.with_offset(offset);
            combine(&mut self.bounds, &b);
            *word_offset = offset;
            offset.x += b.right - b.left;
        }
    }

    fn draw(&self, canvas: &mut Canvas, paint: &Paint) {
        canvas.save();
        canvas.translate((-self.bounds.left, -self.bounds.top));
        for (word, _, position) in &self.words {
            canvas.save();
            canvas.translate(*position);
            canvas.draw_path(&word.path, &paint);
            canvas.restore();
        }
        canvas.restore();
    }
}

// TODO: promote this to an utils function
fn combine(a: &mut Rect, b: &Rect) {
    a.left = a.left.min(b.left);
    a.right = a.right.max(b.right);
    a.top = a.top.min(b.top);
    a.bottom = a.bottom.max(b.bottom);
}
