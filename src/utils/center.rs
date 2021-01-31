use crate::skia::Point;

use super::bottom_right::BottomRight;

pub trait Center {
    fn center(&self) -> Point;
}

impl<T: BottomRight> Center for T {
    fn center(&self) -> Point {
        self.bottom_right() * 0.5
    }
}
