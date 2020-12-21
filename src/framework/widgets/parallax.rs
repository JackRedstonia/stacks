use super::{LayoutSize, Widget};
use crate::game::{Canvas, InputEvent, State};
use crate::skia;
use skia::{scalar, Matrix, Point, Rect, Size};
use skulpin_renderer_winit::winit::dpi::LogicalPosition;

pub struct Parallax<T: Widget> {
    pub inner: T,
    pub last_mouse_position: Point,
    pub interpolated_mouse_position: Point,
}

impl<T: Widget> Parallax<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            last_mouse_position: (0.0, 0.0).into(),
            interpolated_mouse_position: (0.0, 0.0).into(),
        }
    }

    fn calc_parallax(&self) -> Matrix {
        Matrix::translate(self.interpolated_mouse_position * 0.05)
    }

    fn interpolate_parallax(&mut self, factor: scalar) {
        let diff = self.last_mouse_position - self.interpolated_mouse_position;
        if diff.distance_to_origin() < 1.0 {
            self.interpolated_mouse_position = self.last_mouse_position;
        } else {
            self.interpolated_mouse_position += diff * factor;
        }
    }
}

impl<T: Widget> Widget for Parallax<T> {
    fn update(&mut self) {
        self.inner.update();
    }

    fn input(&mut self, event: &InputEvent, size: Size) -> bool {
        if let InputEvent::MouseMove(LogicalPosition { x, y }) = event {
            self.last_mouse_position = (*x, *y).into();
        }
        // TODO: test this. might be a soundness hole, ngl
        self.calc_parallax().invert().map_or(false, |m| {
            event.reverse_map_position(m).map_or(false, |event| {
                let (rect, _) = m.map_rect(Rect::from_size(size));
                self.inner.input(&event, rect.size())
            })
        })
    }

    fn size(&mut self) -> LayoutSize {
        self.inner.size()
    }

    fn draw(&mut self, canvas: &mut Canvas, size: Size) {
        self.interpolate_parallax(State::last_update_time_draw().as_secs_f32() * 20.0);
        canvas.save();
        canvas.concat(self.calc_parallax());
        self.inner.draw(canvas, size);
        canvas.restore();
    }
}
