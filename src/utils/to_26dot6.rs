pub trait To26Dot6 {
    fn to_26dot6(self) -> isize;
}

impl To26Dot6 for f32 {
    fn to_26dot6(self) -> isize {
        (self * 64.0) as _
    }
}

impl To26Dot6 for f64 {
    fn to_26dot6(self) -> isize {
        (self * 64.0) as _
    }
}
