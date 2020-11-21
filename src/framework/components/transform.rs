use super::{Component, LayoutSize};
use crate::game::{Canvas, InputEvent, InputState, TimeState};
use crate::skia;
use skia::{Matrix, Rect, Size};

pub struct Transform<T: Component> {
    pub inner: T,
    pub matrix: Matrix,
}

impl<T: Component> Component for Transform<T> {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState) {
        self.inner.update(input_state, time_state);
    }

    fn input(
        &mut self,
        input_state: &InputState,
        time_state: &TimeState,
        event: &InputEvent,
        size: Size,
    ) -> bool {
        // TODO: test this. might be a soundness hole, ngl
        self.matrix.invert().map_or(false, |m| {
            event.reverse_map_position(m).map_or(false, |event| {
                let (rect, _) = m.map_rect(Rect::from_size(size));
                self.inner
                    .input(input_state, time_state, &event, rect.size())
            })
        })
    }

    fn size(&mut self, input_state: &InputState, time_state: &TimeState) -> LayoutSize {
        // if let Some((rect, _)) = self.matrix.invert().map(|m| m.map_rect(Rect::from_size(size))) {
        //     canvas.save();
        //     canvas.concat(self.matrix);
        //     self.inner.draw(input_state, time_state, canvas, rect.size());
        //     canvas.restore();
        // }
        if let Some(matrix) = self.matrix.invert() {
            self.inner.size(input_state, time_state).map(matrix)
        } else {
            LayoutSize::ZERO
        }
    }

    fn draw(
        &mut self,
        input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut Canvas,
        size: Size,
    ) {
        if let Some((rect, _)) = self
            .matrix
            .invert()
            .map(|m| m.map_rect(Rect::from_size(size)))
        {
            canvas.save();
            canvas.concat(self.matrix);
            self.inner
                .draw(input_state, time_state, canvas, rect.size());
            canvas.restore();
        }
    }
}
