use crate::prelude::*;
use game::{InputEvent, State, ID};
use skia::{scalar, Canvas, Matrix, Point, Size};

pub struct Parallax<T: Widget> {
    pub inner: Wrap<T>,
    size: Size,
    pub last_mouse_position: Point,
    pub interpolated_mouse_position: Point,
}

impl<T: Widget> Parallax<T> {
    pub fn new(inner: impl Into<Wrap<T>>) -> Self {
        Self {
            inner: inner.into(),
            size: Size::new_empty(),
            last_mouse_position: (0.0, 0.0).into(),
            interpolated_mouse_position: (0.0, 0.0).into(),
        }
    }

    fn calc_parallax(&self) -> Matrix {
        Matrix::translate((self.interpolated_mouse_position - (self.size / 2.0)) * 0.05)
    }

    fn interpolate_mouse(&mut self, factor: scalar) -> bool {
        let diff = self.last_mouse_position - self.interpolated_mouse_position;
        let snap = diff.distance_to_origin() < 1.0;
        if snap {
            self.interpolated_mouse_position = self.last_mouse_position;
        } else {
            self.interpolated_mouse_position += diff * factor;
        }
        snap
    }
}

impl<T: Widget> Widget for Parallax<T> {
    fn update(&mut self, _wrap: &mut WrapState) {
        self.inner.update();
    }

    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent) -> bool {
        if let InputEvent::MouseMove(p) = event {
            self.last_mouse_position = *p;
        }
        let m = self.calc_parallax();
        event
            .reverse_map_position(m)
            .map_or(false, |event| self.inner.input(&event))
    }

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        self.inner.size()
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
        self.inner.set_size(size);
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        let p = self.calc_parallax();
        if !self.interpolate_mouse(State::last_update_time_draw().as_secs_f32() * 60.0) {
            // `interpolate_mouse` has reported that the child's position has
            // moved significantly. This means we need to re-emit MouseMove,
            // as from its perspective, the mouse really has moved.
            if let Some(p_inv) = p.invert() {
                self.inner.input(&InputEvent::MouseMove(
                    p_inv.map_point(self.last_mouse_position),
                ));
            }
        }
        canvas.save();
        canvas.concat(&p);
        self.inner.draw(canvas);
        canvas.restore();
    }

    fn get(&mut self, _wrap: &mut WrapState, id: ID) -> Option<(&mut dyn Widget, &mut WrapState)> {
        self.inner.get(id)
    }
}
