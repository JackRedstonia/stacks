use skulpin_renderer_winit::winit::dpi::LogicalPosition;

use crate::skia::{scalar, IRect, ISize, Point, Rect, Size};

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

pub trait ToPoint {
    fn to_point(&self) -> Point;
}

impl ToPoint for LogicalPosition<scalar> {
    fn to_point(&self) -> Point {
        Point::new(self.x, self.y)
    }
}
