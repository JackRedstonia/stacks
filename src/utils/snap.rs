use crate::skia::scalar;

pub trait Snap {
    fn snap(self, origin: Self, precision: Self) -> Self;
}

impl Snap for scalar {
    fn snap(self, origin: Self, precision: Self) -> Self {
        origin + ((self - origin) / precision).round() / (1.0 / precision)
    }
}

impl Snap for f64 {
    fn snap(self, origin: Self, precision: Self) -> Self {
        origin + ((self - origin) / precision).round() / (1.0 / precision)
    }
}
