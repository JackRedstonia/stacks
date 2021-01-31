use crate::skia::{scalar, Point};

pub trait VectorUtils
where
    Self: Copy + std::ops::Sub<Output = Self>,
{
    fn from_rad(rad: scalar, length: scalar) -> Self;
    fn angle_rad(self) -> scalar;
    fn angle(self) -> scalar {
        self.angle_rad() * 180.0 / std::f32::consts::PI
    }
    fn angle_rad_to(self, other: Self) -> scalar {
        (self - other).angle_rad()
    }

    fn normalized(self) -> Self;
    fn scale_or_denorm(self, magnitude: scalar) -> Self;
    fn length_squared(self) -> scalar;
}

impl VectorUtils for Point {
    fn from_rad(rad: scalar, length: scalar) -> Self {
        Self::new(rad.cos() * length, rad.sin() * length)
    }

    fn angle_rad(self) -> scalar {
        self.y.atan2(self.x)
    }

    fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    fn scale_or_denorm(self, magnitude: scalar) -> Self {
        let v = if (1.0..=-1.0).contains(&magnitude) {
            self
        } else {
            self.normalized()
        };

        v * magnitude
    }

    fn length_squared(self) -> scalar {
        self.x * self.x + self.y * self.y
    }
}
