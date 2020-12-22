use super::{LayoutSize, Widget, WidgetWrapper, WrapperState};
use crate::game::{Canvas, InputEvent};
use crate::skia;
use skia::{Matrix, Rect, Size};

pub struct Transform<T: Widget> {
    pub inner: WidgetWrapper<T>,
    size: LayoutSize,
    pub matrix: Matrix,
}

impl<T: Widget> Transform<T> {
    pub fn new(inner: WidgetWrapper<T>, matrix: Matrix) -> Self {
        Self {
            inner,
            size: LayoutSize::ZERO,
            matrix,
        }
    }
}

impl<T: Widget> Widget for Transform<T> {
    fn update(&mut self, wrap: &mut WrapperState) {
        self.inner.update();
    }

    fn input(&mut self, event: &InputEvent, size: Size) -> bool {
        // TODO: test this. might be a soundness hole, ngl
        self.matrix.invert().map_or(false, |m| {
            event.reverse_map_position(m).map_or(false, |event| {
                let (rect, _) = m.map_rect(Rect::from_size(size));
                self.inner.input(&event, rect.size())
            })
        })
    }

    fn size(&mut self) -> LayoutSize {
        self.size = self.inner.size();
        self.size.map(self.matrix)
    }

    fn draw(&mut self, canvas: &mut Canvas, size: Size) {
        canvas.save();
        canvas.concat(self.matrix);
        self.inner.draw(canvas, self.size.layout_one(size));
        canvas.restore();
    }
}
