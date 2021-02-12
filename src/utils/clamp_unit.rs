pub trait ClampUnit {
    fn clamp_unit(self) -> Self;
}

impl ClampUnit for f32 {
    #[inline]
    fn clamp_unit(self) -> Self {
        self.clamp(0.0, 1.0)
    }
}

impl ClampUnit for f64 {
    #[inline]
    fn clamp_unit(self) -> Self {
        self.clamp(0.0, 1.0)
    }
}
