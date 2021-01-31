use crate::skia::{scalar, Vector};

pub trait Easing
where
    Self: Sized,
{
    fn ease_in_pow(self, pow: i32) -> Self;
    fn ease_in_out_pow(self, pow: i32) -> Self;
    fn ease_out_pow(self, pow: i32) -> Self;

    fn ease_in_sine(self) -> Self;
    fn ease_in_out_sine(self) -> Self;
    fn ease_out_sine(self) -> Self;

    #[inline(always)]
    fn ease_in_quad(self) -> Self {
        self.ease_in_pow(2)
    }

    #[inline(always)]
    fn ease_in_cubic(self) -> Self {
        self.ease_in_pow(3)
    }

    #[inline(always)]
    fn ease_in_quart(self) -> Self {
        self.ease_in_pow(4)
    }

    #[inline(always)]
    fn ease_in_quint(self) -> Self {
        self.ease_in_pow(5)
    }

    #[inline(always)]
    fn ease_in_out_quad(self) -> Self {
        self.ease_in_out_pow(2)
    }

    #[inline(always)]
    fn ease_in_out_cubic(self) -> Self {
        self.ease_in_out_pow(3)
    }

    #[inline(always)]
    fn ease_in_out_quart(self) -> Self {
        self.ease_in_out_pow(4)
    }

    #[inline(always)]
    fn ease_in_out_quint(self) -> Self {
        self.ease_in_out_pow(5)
    }

    #[inline(always)]
    fn ease_out_quad(self) -> Self {
        self.ease_out_pow(2)
    }

    #[inline(always)]
    fn ease_out_cubic(self) -> Self {
        self.ease_out_pow(3)
    }

    #[inline(always)]
    fn ease_out_quart(self) -> Self {
        self.ease_out_pow(4)
    }

    #[inline(always)]
    fn ease_out_quint(self) -> Self {
        self.ease_out_pow(5)
    }
}

impl Easing for scalar {
    #[inline(always)]
    fn ease_in_pow(self, pow: i32) -> Self {
        self.powi(pow)
    }

    #[inline(always)]
    fn ease_in_out_pow(self, pow: i32) -> Self {
        if self < 0.5 {
            self.powi(pow) * (2f32).powi(pow - 1)
        } else {
            1.0 - (-2.0 * self + 2.0).powi(pow) * 0.5
        }
    }

    #[inline(always)]
    fn ease_out_pow(self, pow: i32) -> Self {
        1.0 - (1.0 - self).powi(pow)
    }

    #[inline(always)]
    fn ease_in_sine(self) -> Self {
        1.0 - (self * std::f32::consts::FRAC_PI_2).cos()
    }

    #[inline(always)]
    fn ease_in_out_sine(self) -> Self {
        -((self * std::f32::consts::PI).cos() - 1.0) * 0.5
    }

    #[inline(always)]
    fn ease_out_sine(self) -> Self {
        (self * std::f32::consts::FRAC_PI_2).sin()
    }
}

impl Easing for Vector {
    fn ease_in_pow(self, pow: i32) -> Self {
        Self::new(self.x.ease_in_pow(pow), self.y.ease_in_pow(pow))
    }

    fn ease_in_out_pow(self, pow: i32) -> Self {
        Self::new(self.x.ease_in_out_pow(pow), self.y.ease_in_out_pow(pow))
    }

    fn ease_out_pow(self, pow: i32) -> Self {
        Self::new(self.x.ease_out_pow(pow), self.y.ease_out_pow(pow))
    }

    fn ease_in_sine(self) -> Self {
        Self::new(self.x.ease_in_sine(), self.y.ease_in_sine())
    }

    fn ease_in_out_sine(self) -> Self {
        Self::new(self.x.ease_in_out_sine(), self.y.ease_in_out_sine())
    }

    fn ease_out_sine(self) -> Self {
        Self::new(self.x.ease_out_sine(), self.y.ease_out_sine())
    }
}