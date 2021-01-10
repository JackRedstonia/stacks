use crate::skia::{IRect, ISize, Paint, Point, Rect, Size};

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

pub trait AntiAlias {
    fn with_anti_alias(self, anti_alias: bool) -> Self;
}

impl AntiAlias for Paint {
    fn with_anti_alias(mut self, anti_alias: bool) -> Self {
        self.set_anti_alias(anti_alias);
        self
    }
}
