use super::{Font, FontStyle};
use crate::prelude::*;

use std::mem::transmute;

use skia::font_style::{Slant, Weight, Width};
use skia::{Font as SkFont, FontMgr, FontStyle as SkFontStyle, Typeface};

pub struct Fonts {
    resource: ResourceHoster<FontResource>,
}

impl Fonts {
    pub fn new() -> Wrap<Self> {
        Self {
            resource: FontResource::new(),
        }
        .into()
    }
}

impl Widget for Fonts {
    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack) {
        stack.push(self.resource.new_user());
        for i in state.children() {
            i.load(stack);
        }
        stack.pop::<ResourceUser<FontResource>>();
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        state
            .children()
            .next()
            .map(|e| e.input(event))
            .unwrap_or(false)
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        state
            .children()
            .next()
            .map(|e| e.size())
            .unwrap_or_default()
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        if let Some(child) = state.child() {
            child.set_size(size);
        }
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        if let Some(child) = state.child() {
            child.draw(canvas);
        }
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
            default: FontSet::new("IBM Plex Sans"),
            fallback_ja: FontSet::new(&ja),
            fallback_vn: FontSet::new(&vn),
        })
    }

    pub fn resolve(&self, font: Font, style: FontStyle, size: Option<scalar>) -> Vec<SkFont> {
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

impl FontSet {
    fn new(family_name: &str) -> Self {
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
            size.unwrap_or(16.0),
        )
    }
}
