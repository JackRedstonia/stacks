use super::{Font, FontStyle};
use crate::prelude::*;

use std::mem::transmute;

use skia::font_style::{Slant, Weight, Width};
use skia::{Data, Font as SkFont, FontMgr, FontStyle as SkFontStyle, Typeface};

pub struct Fonts<T: Widget + ?Sized> {
    child: Wrap<T>,
    resource: ResourceHoster<FontResource>,
}

impl<T: Widget + ?Sized> Fonts<T> {
    pub fn new(child: Wrap<T>) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            child,
            resource: FontResource::new(),
        }
        .into()
    }
}

impl<T: Widget + ?Sized> Widget for Fonts<T> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        stack.push(self.resource.new_user());
        self.child.load(stack);
        stack.pop::<ResourceUser<FontResource>>();
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.child.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        self.child.input(event)
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        self.child.size()
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.child.set_size(size);
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        self.child.draw(canvas);
    }
}

pub struct FontResource {
    default: FontSet,
    fallback_ja: FontSet,
    fallback_vn: FontSet,
}

impl FontResource {
    pub fn new() -> ResourceHoster<Self> {
        let mgr = FontMgr::new();

        // We're mostly just probing system fonts here based on whether they
        // support Japanese and Vietnamese, falling back to Noto Sans because
        // that's what Skia comes with.
        // The unsafe code here is because skia-safe (heheh, "safe"...) takes
        // i32 instead of char for Skia's Unichar.

        let ja = unsafe { transmute('あ') };
        let ja = mgr.match_family_style_character(
            "",
            SkFontStyle::default(),
            &["ja"],
            ja,
        );
        let ja = ja
            .map(|e| e.family_name())
            .unwrap_or("Noto Sans".to_owned());

        let vn = unsafe { transmute('ố') };
        let vn = mgr.match_family_style_character(
            "",
            SkFontStyle::default(),
            &["vn"],
            vn,
        );
        let vn = vn
            .map(|e| e.family_name())
            .unwrap_or("Noto Sans".to_owned());

        ResourceHoster::new(Self {
            default: FontSet::new(),
            fallback_ja: FontSet::from_type_name(&ja),
            fallback_vn: FontSet::from_type_name(&vn),
        })
    }

    pub fn resolve(
        &self,
        font: Font,
        style: FontStyle,
        size: Option<scalar>,
    ) -> Vec<SkFont> {
        let f = match font {
            Font::Default => &self.default,
        };
        vec![
            f.get(style, size),
            self.fallback_ja.get(style, size),
            self.fallback_vn.get(style, size),
        ]
    }
}

struct FontSet {
    regular: Typeface,
    bold: Typeface,
    italic: Typeface,
    bold_italic: Typeface,
}

macro_rules! font_bytes {
    ($s:expr) => {
        include_bytes!(concat!(
            "../../../../resources/fonts/IBMPlexSans-",
            $s,
            ".ttf"
        ))
    };
}

impl FontSet {
    fn new() -> Self {
        let regular = unsafe { Data::new_bytes(font_bytes!("Regular")) };
        let bold = unsafe { Data::new_bytes(font_bytes!("Medium")) };
        let italic = unsafe { Data::new_bytes(font_bytes!("Italic")) };
        let bold_italic = unsafe { Data::new_bytes(font_bytes!("MediumItalic")) };
        Self {
            regular: Typeface::from_data(regular, None).unwrap(),
            bold: Typeface::from_data(bold, None).unwrap(),
            italic: Typeface::from_data(italic, None).unwrap(),
            bold_italic: Typeface::from_data(bold_italic, None).unwrap(),
        }
    }

    fn from_type_name(family_name: &str) -> Self {
        Self {
            regular: Typeface::from_name(
                family_name,
                SkFontStyle::new(Weight::MEDIUM, Width::NORMAL, Slant::Upright),
            )
            .unwrap(),
            bold: Typeface::from_name(
                family_name,
                SkFontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Upright),
            )
            .unwrap(),
            italic: Typeface::from_name(
                family_name,
                SkFontStyle::new(Weight::MEDIUM, Width::NORMAL, Slant::Italic),
            )
            .unwrap(),
            bold_italic: Typeface::from_name(
                family_name,
                SkFontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Italic),
            )
            .unwrap(),
        }
    }

    fn get(&self, style: FontStyle, size: Option<scalar>) -> SkFont {
        SkFont::new(
            match style {
                FontStyle::Regular => &self.regular,
                FontStyle::Bold => &self.bold,
                FontStyle::Italic => &self.italic,
                FontStyle::BoldItalic => &self.bold_italic,
            },
            size.unwrap_or(13.5),
        )
    }
}
