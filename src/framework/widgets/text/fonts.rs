use super::{Font, FontStyle};
use crate::prelude::*;

use skia::font_style::{Slant, Weight, Width};
use skia::{Font as SkFont, FontStyle as SkFontStyle, Typeface};

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
}

impl FontResource {
    pub fn new() -> ResourceHoster<Self> {
        ResourceHoster::new(Self {
            default: FontSet::new("IBM Plex Sans"),
        })
    }

    pub fn resolve(&self, font: Font, style: FontStyle) -> SkFont {
        let f = match font {
            Font::Default => &self.default,
        };
        f.get(style)
    }
}

struct FontSet {
    size: scalar,
    regular: Typeface,
    bold: Typeface,
    italic: Typeface,
    bold_italic: Typeface,
}

impl FontSet {
    fn new(family_name: &str) -> Self {
        Self {
            size: 16.0,
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

    fn get(&self, style: FontStyle) -> SkFont {
        SkFont::new(
            match style {
                FontStyle::Regular => &self.regular,
                FontStyle::Bold => &self.bold,
                FontStyle::Italic => &self.italic,
                FontStyle::BoldItalic => &self.bold_italic,
            },
            self.size,
        )
    }
}
