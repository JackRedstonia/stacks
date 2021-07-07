pub use freetype::Error as FTError;

use super::FontStyle;
use crate::game::ID;
use crate::prelude::*;

use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::rc::Rc;

use freetype::face::LoadFlag;
use freetype::ffi::FT_Size_Metrics;
use freetype::outline::Curve;

macro_rules! include_font {
    ($r:expr, $s:expr, $n:expr, $t:expr, $z:expr) => {
        $r.register_face_memory(
            include_bytes!(concat!(
                "../../../../resources/fonts/FiraSans-",
                $s,
                ".otf"
            ))
            .to_vec(),
            $n,
            $t,
            $z,
        )?;
    };
}

pub type FontName = Cow<'static, str>;

pub struct Fonts<T: Widget + ?Sized> {
    res: ResourceHoster<FontResource>,
    inner: Wrap<T>,
}

impl<T: Widget + ?Sized> Fonts<T> {
    pub fn new(inner: Wrap<T>) -> Result<Wrap<Self>, FTError> {
        let n = Cow::Borrowed("Fira Sans");
        let res = FontResource::new(n.clone())?;
        let mut r = res.access_mut();
        let z = 13.5f32.to_26dot6();
        include_font!(r, "Regular", n.clone(), FontStyle::Regular, z);
        include_font!(r, "Medium", n.clone(), FontStyle::Medium, z);
        include_font!(r, "Bold", n.clone(), FontStyle::Bold, z);
        include_font!(r, "Italic", n.clone(), FontStyle::Italic, z);
        include_font!(r, "MediumItalic", n.clone(), FontStyle::MediumItalic, z);
        include_font!(r, "BoldItalic", n, FontStyle::BoldItalic, z);
        drop(r);
        Ok(Self { res, inner }.into())
    }
}

impl<T: Widget + ?Sized> Widget for Fonts<T> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        stack.push::<ResourceUser<FontResource>>(self.res.new_user());
        self.inner.load(stack);
        stack.pop::<ResourceUser<FontResource>>();
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.inner.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        self.inner.input(event)
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        self.inner.size()
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.inner.set_size(size);
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        self.inner.draw(canvas);
    }
}

pub struct FontResource {
    lib: freetype::Library,
    cache: FontCache,
    default_face: FontName,
}

impl FontResource {
    fn new(default_face: FontName) -> Result<ResourceHoster<Self>, FTError> {
        Ok(ResourceHoster::new(Self {
            lib: freetype::Library::init()?,
            cache: FontCache::new(),
            default_face,
        }))
    }

    pub fn register_face(
        &mut self,
        p: impl AsRef<OsStr>,
        name: FontName,
        style: FontStyle,
        default_size: isize,
    ) -> Result<ID, FTError> {
        let id = self.cache.register_face(
            &self.lib,
            p,
            name,
            style,
            default_size,
        )?;
        self.register_size(id, default_size).unwrap();
        Ok(id)
    }

    pub fn register_face_memory(
        &mut self,
        buf: impl Into<Rc<Vec<u8>>>,
        name: FontName,
        style: FontStyle,
        default_size: isize,
    ) -> Result<ID, FTError> {
        let id = self.cache.register_face_memory(
            &self.lib,
            buf,
            name,
            style,
            default_size,
        )?;
        self.register_size(id, default_size).unwrap();
        Ok(id)
    }

    pub fn register_size(&mut self, face: ID, size: isize) -> Result<(), ()> {
        self.cache.register_size(face, size)
    }

    pub fn get_face(
        &self,
        name: Option<&FontName>,
        style: FontStyle,
    ) -> Option<ID> {
        match name {
            Some(c) => self.cache.get_face(c, style),
            None => self
                .cache
                .face_ids
                .get(&(Cow::Borrowed(&self.default_face), style))
                .map(|e| *e),
        }
    }

    pub fn get_font(
        &mut self,
        face: ID,
        size: Option<isize>,
    ) -> Option<&mut CachedFont> {
        if let Some(size) = size {
            self.register_size(face, size).ok()?;
        }
        self.cache.get_font(face, size)
    }
}

struct FontCache {
    face_ids: HashMap<(FontName, FontStyle), ID>,
    faces: HashMap<ID, (freetype::Face, isize)>,
    sized_fonts: HashMap<(ID, isize), CachedFont>,
}

impl FontCache {
    const COMMON_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890`-=[]\\;',./~!@#$%^&*()_+{}|:\"<>? ";

    fn new() -> Self {
        Self {
            face_ids: HashMap::new(),
            faces: HashMap::new(),
            sized_fonts: HashMap::new(),
        }
    }

    fn register_face(
        &mut self,
        lib: &freetype::Library,
        p: impl AsRef<OsStr>,
        name: FontName,
        style: FontStyle,
        default_size: isize,
    ) -> Result<ID, FTError> {
        let e = (name, style);
        if let Some(id) = self.face_ids.get(&e) {
            return Ok(*id);
        }
        let face = lib.new_face(p, 0)?;
        let id = ID::next();
        self.face_ids.insert(e, id);
        self.faces.insert(id, (face, default_size));
        Ok(id)
    }

    fn register_face_memory(
        &mut self,
        lib: &freetype::Library,
        buf: impl Into<Rc<Vec<u8>>>,
        name: FontName,
        style: FontStyle,
        default_size: isize,
    ) -> Result<ID, FTError> {
        let e = (name, style);
        if let Some(id) = self.face_ids.get(&e) {
            return Ok(*id);
        }
        let face = lib.new_memory_face(buf, 0)?;
        let id = ID::next();
        self.face_ids.insert(e, id);
        self.faces.insert(id, (face, default_size));
        Ok(id)
    }

    fn register_size(&mut self, face: ID, size: isize) -> Result<(), ()> {
        if self.sized_fonts.contains_key(&(face, size)) {
            return Ok(());
        }
        let ftface = &self.faces.get(&face).ok_or(())?.0;
        ftface.set_char_size(size, 0, 0, 0).unwrap();
        let mut font = CachedFont {
            size,
            face: ftface.clone(),
            cache: HashMap::new(),
            metrics: FontMetrics::from_ft(ftface.size_metrics().unwrap()),
        };
        for c in Self::COMMON_CHARS.chars() {
            let _ = font.get_char(c);
        }
        self.sized_fonts.insert((face, size), font);
        Ok(())
    }

    fn get_face(&self, name: &FontName, style: FontStyle) -> Option<ID> {
        self.face_ids.get(&(Cow::Borrowed(name), style)).map(|e| *e)
    }

    fn get_font(
        &mut self,
        face: ID,
        size: Option<isize>,
    ) -> Option<&mut CachedFont> {
        let size = match size {
            Some(s) => s,
            None => self
                .faces
                .get(&face)
                .map(|(_, default_size)| *default_size)?,
        };
        self.sized_fonts.get_mut(&(face, size))
    }
}

pub struct CachedFont {
    size: isize,
    face: freetype::Face,
    cache: HashMap<char, (skia::Path, scalar)>,
    pub metrics: FontMetrics,
}

impl CachedFont {
    pub fn get_char(&mut self, c: char) -> (skia::Path, scalar) {
        if let Some(i) = self.cache.get(&c) {
            i.clone()
        } else {
            let (path, right) = self.make_char(c);
            self.cache.insert(c, (path.clone(), right));
            (path, right)
        }
    }

    fn make_char(&mut self, c: char) -> (skia::Path, scalar) {
        self.face.set_char_size(self.size, 0, 0, 0).unwrap();
        self.face
            .load_char(
                c as usize,
                LoadFlag::NO_BITMAP | LoadFlag::TARGET_NORMAL,
            )
            .unwrap();
        let glyph = self.face.glyph();
        let advance = ft_26dot6_to_f32(glyph.advance().x);
        let outline =
            glyph.outline().expect("Non-vector fonts are not supported");
        let mut path = skia::Path::new();
        for contour in outline.contours_iter() {
            path.move_to(ft2sk_vector(*contour.start()));
            for curve in contour {
                match curve {
                    Curve::Line(v) => path.line_to(ft2sk_vector(v)),
                    Curve::Bezier2(a, b) => {
                        path.quad_to(ft2sk_vector(a), ft2sk_vector(b))
                    }
                    Curve::Bezier3(a, b, c) => path.cubic_to(
                        ft2sk_vector(a),
                        ft2sk_vector(b),
                        ft2sk_vector(c),
                    ),
                };
            }
            path.close();
        }
        (path, advance)
    }
}

#[derive(Clone)]
pub struct FontMetrics {
    pub ascent: scalar,
    pub descent: scalar,
    pub height: scalar,
}

impl FontMetrics {
    fn from_ft(metrics: FT_Size_Metrics) -> Self {
        Self {
            ascent: -ft_26dot6_to_f32(metrics.ascender),
            descent: -ft_26dot6_to_f32(metrics.descender),
            height: ft_26dot6_to_f32(metrics.height),
        }
    }
}

fn ft_26dot6_to_f32(d: i64) -> f32 {
    d as f32 / 64.0
}

fn ft2sk_vector(v: freetype::Vector) -> Vector {
    (ft_26dot6_to_f32(v.x), -ft_26dot6_to_f32(v.y)).into()
}
