mod fonts;

pub use fonts::{FontResource, Fonts};

use crate::prelude::*;
use skia::{Canvas, Font as SkFont, GlyphId, Path};

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
    pub layout_size: LayoutSize,
    pub paint: Paint,
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
        text: impl AsRef<str>,
        font: Font,
        style: FontStyle,
        font_size: Option<scalar>,
        paint: Paint,
    ) -> Wrap<Self> {
        let text = text.as_ref();
        Self {
            layout_size: size,
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

    fn shape(&mut self) {
        if let Some(f) = &self.sk_font {
            self.paragraph =
                Some(Paragraph::new(&self.text, f, self.size.width));
        }
    }
}

impl Widget for Text {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some(f) = stack.get::<ResourceUser<FontResource>>() {
            if let Some(f) = f.try_access() {
                self.sk_font = Some(f.resolve(self.font, self.style, self.font_size));
            }
        }
    }

    fn input(&mut self, _state: &mut WidgetState, _event: &InputEvent) -> bool {
        // TODO: this is mostly a placeholder value.
        // I'm pretty sure somebody will have a use for some text to handle click events, that sort of stuff.
        false
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
        self.shape();
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        //
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
    words: Vec<(Word, Vector)>,
    bounds: Rect,
}

impl Paragraph {
    fn new(s: &str, font: &[SkFont], width: scalar) -> Self {
        assert!(!font.is_empty());
        let mut prev = 0;
        let words = unicode_linebreak::linebreaks(s).map(|(e, _)| {
            let r = &s[prev..e];
            prev = e;
            Word::new(r, font)
        });
        let mut out = vec![];
        let mut bounds = Rect::new_empty();
        let mut offset = Vector::default();
        let line = font[0].metrics().0;
        for word in words {
            let nx = offset.x + word.bounds.right - word.bounds.left;
            if nx >= width && offset.x != 0.0 {
                offset = Vector::new(0.0, offset.y + line);
            }
            let b = word.bounds.with_offset(offset);
            combine(&mut bounds, &b);
            out.push((word, offset));
            offset.x += b.right - b.left;
        }
        Self { words: out, bounds }
    }

    fn draw(&self, canvas: &mut Canvas, paint: &Paint) {
        canvas.save();
        canvas.translate((-self.bounds.left, -self.bounds.top));
        for (word, position) in &self.words {
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
