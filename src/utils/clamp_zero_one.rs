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
