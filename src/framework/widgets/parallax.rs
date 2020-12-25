use super::{LayoutSize, Widget, Wrap, WrapState, ID};
use crate::game::{Canvas, InputEvent, State};
use crate::skia;
use skia::{scalar, Matrix, Point, Rect, Size};
use skulpin_renderer_winit::winit::dpi::LogicalPosition;

pub struct Parallax<T: Widget> {
    pub inner: Wrap<T>,
    pub last_mouse_position: Point,
    pub interpolated_mouse_position: Point,
}

impl<T: Widget> Parallax<T> {
    pub fn new(inner: Wrap<T>) -> Self {
        Self {
            inner,
            last_mouse_position: (0.0, 0.0).into(),
            interpolated_mouse_position: (0.0, 0.0).into(),
        }
    }

    fn calc_parallax(&self, size: Size) -> Matrix {
        Matrix::translate((self.interpolated_mouse_position - (size / 2.0)) * 0.05)
    }

    fn interpolate_mouse(&mut self, factor: scalar) {
        let diff = self.last_mouse_position - self.interpolated_mouse_position;
        if diff.distance_to_origin() < 1.0 {
            self.interpolated_mouse_position = self.last_mouse_position;
        } else {
            self.interpolated_mouse_position += diff * factor;
        }
    }
}

impl<T: Widget> Widget for Parallax<T> {
    fn update(&mut self, _wrap: &mut WrapState) {
        self.inner.update();
    }

    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent, size: Size) -> bool {
        if let InputEvent::MouseMove(LogicalPosition { x, y }) = event {
            self.last_mouse_position = (*x, *y).into();
        }
        let m = self.calc_parallax(size);
        event.reverse_map_position(m).map_or(false, |event| {
            let (rect, _) = m.map_rect(Rect::from_size(size));
            self.inner.input(&event, rect.size())
        })
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        self.inner.size()
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas, size: Size) {
        self.interpolate_mouse(State::last_update_time_draw().as_secs_f32() * 20.0);
        canvas.save();
        canvas.concat(self.calc_parallax(size));
        self.inner.draw(canvas, size);
        canvas.restore();
    }

    fn get(&mut self, _wrap: &mut WrapState, id: ID) -> Option<(&mut dyn Widget, &mut WrapState)> {
        self.inner.get(id)
    }
}
