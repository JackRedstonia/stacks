use crate::prelude::*;

pub struct Transform {
    pub matrix: Matrix,
}

impl Transform {
    pub fn new(matrix: Matrix) -> Wrap<Self> {
        FrameworkState::request_load();
        Self { matrix }.into()
    }
}

impl Widget for Transform {
    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        state
            .child()
            .map(|child| {
                event
                    .reverse_map_position(self.matrix)
                    .map_or(false, |event| child.input(&event))
            })
            .unwrap_or(false)
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        state
            .child()
            .map(|child| {
                let s = child.size();
                (s.0.map(self.matrix), s.1)
            })
            .unwrap_or_default()
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        if let Some(m) = self.matrix.invert() {
            if let Some(child) = state.child() {
                let (rect, _) = m.map_rect(Rect::from_size(size));
                child.set_size(rect.size());
            }
        }
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        if let Some(child) = state.child() {
            canvas.save();
            canvas.concat(&self.matrix);
            child.draw(canvas);
            canvas.restore();
        }
    }
}
