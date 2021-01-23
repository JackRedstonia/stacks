use crate::skia::{
    scalar, Color4f, ColorSpace, IRect, ISize, Paint, PaintStyle, Point, Rect, Size,
};

pub trait BottomRight {
    fn bottom_right(&self) -> Point;
}

impl BottomRight for Rect {
    fn bottom_right(&self) -> Point {
        Point::new(self.right, self.bottom)
    }
}

impl BottomRight for IRect {
    fn bottom_right(&self) -> Point {
        Point::new(self.right as f32, self.bottom as f32)
    }
}

impl BottomRight for Size {
    fn bottom_right(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BottomRight for ISize {
    fn bottom_right(&self) -> Point {
        Point::new(self.width as f32, self.height as f32)
    }
}

pub trait Center {
    fn center(&self) -> Point;
}

impl Center for Size {
    fn center(&self) -> Point {
        self.bottom_right() * 0.5
    }
}

impl Center for ISize {
    fn center(&self) -> Point {
        self.bottom_right() * 0.5
    }
}

pub trait PaintUtils
where
    Self: Sized,
{
    // TODO: require with_color4f_and_space.
    // Add a default with_color4f, which passes a None into color_space.
    // Currently blocked on https://github.com/rust-skia/rust-skia/issues/456

    // Quick constructors
    fn new_color4f_colorspaced(r: f32, g: f32, b: f32, a: f32, s: Option<&ColorSpace>) -> Self;

    fn new_color4f(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new_color4f_colorspaced(r, g, b, a, None)
    }

    // Builder-like mutating methods
    fn with_anti_alias(self, anti_alias: bool) -> Self;
    fn with_stroke(self, stroke: bool) -> Self;
    fn with_stroke_width(self, width: scalar) -> Self;
    fn with_style(self, style: PaintStyle) -> Self;
    fn with_alpha(self, alpha: scalar) -> Self;

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
}

impl PaintUtils for Paint {
    fn new_color4f_colorspaced(r: f32, g: f32, b: f32, a: f32, space: Option<&ColorSpace>) -> Self {
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
}

pub trait ClampZeroOne {
    fn clamp_zero_one(self) -> Self;
}

impl ClampZeroOne for f32 {
    fn clamp_zero_one(self) -> Self {
        self.min(1.0).max(0.0)
    }
}

impl ClampZeroOne for f64 {
    fn clamp_zero_one(self) -> Self {
        self.min(1.0).max(0.0)
    }
}
