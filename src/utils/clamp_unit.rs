pub trait ClampUnit {
    fn clamp_unit(self) -> Self;
}

impl ClampUnit for f32 {
    fn clamp_unit(self) -> Self {
        self.min(1.0).max(0.0)
    }
}

impl ClampUnit for f64 {
    fn clamp_unit(self) -> Self {
        self.min(1.0).max(0.0)
    }
}
