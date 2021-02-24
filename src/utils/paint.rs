use crate::skia::{scalar, Color4f, ColorSpace, Paint, PaintStyle, Shader};

pub trait PaintUtils
where
    Self: Sized,
{
    // TODO: require with_color4f_and_space.
    // Add a default with_color4f, which passes a None into color_space.
    // Currently blocked on https://github.com/rust-skia/rust-skia/issues/456

    // Quick constructors
    fn new_color4f_colorspaced(
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        s: Option<&ColorSpace>,
    ) -> Self;

    fn new_color4f(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new_color4f_colorspaced(r, g, b, a, None)
    }

    // Builder-like mutating methods
    fn with_anti_alias(self, anti_alias: bool) -> Self;
    fn with_stroke(self, stroke: bool) -> Self;
    fn with_stroke_width(self, width: scalar) -> Self;
    fn with_style(self, style: PaintStyle) -> Self;
    fn with_alpha(self, alpha: scalar) -> Self;
    fn with_shader(self, shader: impl Into<Option<Shader>>) -> Self;
    fn with_dither(self, dither: bool) -> Self;

    fn anti_alias(self) -> Self {
        self.with_anti_alias(true)
    }

    fn no_anti_alias(self) -> Self {
        self.with_anti_alias(false)
    }

    fn stroke(self) -> Self {
        self.with_stroke(true)
    }

    fn no_stroke(self) -> Self {
        self.with_stroke(false)
    }

    fn fill_style(self) -> Self {
        self.with_style(PaintStyle::Fill)
    }

    fn stroke_style(self) -> Self {
        self.with_style(PaintStyle::Stroke)
    }

    fn stroke_and_fill_style(self) -> Self {
        self.with_style(PaintStyle::StrokeAndFill)
    }

    fn shade(self, shader: impl Into<Shader>) -> Self {
        self.with_shader(Some(shader.into()))
    }

    fn no_shader(self) -> Self {
        self.with_shader(None)
    }

    fn dither(self) -> Self {
        self.with_dither(true)
    }

    fn no_dither(self) -> Self {
        self.with_dither(false)
    }
}

impl PaintUtils for Paint {
    fn new_color4f_colorspaced(
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        space: Option<&ColorSpace>,
    ) -> Self {
        Self::new(Color4f::new(r, g, b, a), space)
    }

    fn with_anti_alias(mut self, anti_alias: bool) -> Self {
        self.set_anti_alias(anti_alias);
        self
    }

    fn with_stroke(mut self, stroke: bool) -> Self {
        self.set_stroke(stroke);
        self
    }

    fn with_stroke_width(mut self, width: scalar) -> Self {
        self.set_stroke_width(width);
        self
    }

    fn with_style(mut self, style: PaintStyle) -> Self {
        self.set_style(style);
        self
    }

    fn with_alpha(mut self, alpha: scalar) -> Self {
        self.set_alpha_f(alpha);
        self
    }

    fn with_shader(mut self, shader: impl Into<Option<Shader>>) -> Self {
        self.set_shader(shader);
        self
    }

    fn with_dither(mut self, dither: bool) -> Self {
        self.set_dither(dither);
        self
    }
}
