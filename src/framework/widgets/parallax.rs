use crate::prelude::*;

/// Container that moves its contents to the mouse cursor.
///
/// Should only be used for backgrounds, as putting any UI element behind this
/// will certainly hinder accessiblity, especially for those with trackpads.
pub struct Parallax<T: Widget> {
    pub inner: Wrap<T>,
    size: Size,
    interpolated_mouse_position: Vector,
}

impl<T: Widget> Parallax<T> {
    pub fn new(inner: impl Into<Wrap<T>>) -> Self {
        FrameworkState::request_load();
        Self {
            inner: inner.into(),
            size: Size::new_empty(),
            interpolated_mouse_position: (0.0, 0.0).into(),
        }
    }

    fn calc_parallax(&self) -> Matrix {
        let center = self.size / 2.0;
        let offset = self.interpolated_mouse_position - center;
        Matrix::translate(offset * 0.05)
    }

    fn interpolate_mouse(&mut self) {
        let factor = State::last_update_time_draw().as_secs_f32() * 60.0;
        let position = State::mouse_position();
        let diff = position - self.interpolated_mouse_position;
        if diff.distance_to_origin() < 1.0 {
            // The difference between the interpolated position and the current
            // position is less than 1px. Snap to the position instead to
            // prevent floating point gotchas.
            self.interpolated_mouse_position = position;
        } else {
            self.interpolated_mouse_position += diff * factor;
        }
    }
}

impl<T: Widget> Widget for Parallax<T> {
    fn load(&mut self, _wrap: &mut WidgetState, stack: &mut ResourceStack) {
        self.inner.load(stack);
    }

    fn update(&mut self, _wrap: &mut WidgetState) {
        self.inner.update();
    }

    fn input(&mut self, _wrap: &mut WidgetState, event: &InputEvent) -> bool {
        let m = self.calc_parallax();
        event
            .reverse_map_position(m)
            .map_or(false, |event| self.inner.input(&event))
    }

    fn size(&mut self, _wrap: &mut WidgetState) -> (LayoutSize, bool) {
        self.inner.size()
    }

    fn set_size(&mut self, _wrap: &mut WidgetState, size: Size) {
        self.size = size;
        self.inner.set_size(size);
    }

    fn draw(&mut self, _wrap: &mut WidgetState, canvas: &mut Canvas) {
        self.interpolate_mouse();
        canvas.save();
        canvas.concat(&self.calc_parallax());
        self.inner.draw(canvas);
        canvas.restore();
    }
}
