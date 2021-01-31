use crate::skia::{IRect, ISize, Point, Rect, Size};

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
