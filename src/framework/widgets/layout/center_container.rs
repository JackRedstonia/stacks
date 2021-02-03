use crate::prelude::*;

pub struct CenterContainer {
    size: Size,
    child_layout_size: LayoutSize,
    matrix: Matrix,
}

impl CenterContainer {
    pub fn new() -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            child_layout_size: LayoutSize::ZERO,
            matrix: Matrix::default(),
            size: Size::default(),
        }
        .into()
    }
}

impl Widget for CenterContainer {
    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        state
            .child()
            .map(|child| {
                event
                    .reverse_map_position(self.matrix)
                    .map(|e| child.input(&e))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        state
            .child()
            .map(|child| {
                let (child_size, changed) = child.size();
                self.child_layout_size = child_size;
                (child_size.expand_width().expand_height(), changed)
            })
            .unwrap_or_default()
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        if let Some(child) = state.child() {
            self.size = size;
            let child_size = self.child_layout_size.layout_one(size);
            let offset =
                (size.bottom_right() - child_size.bottom_right()) * 0.5;
            self.matrix = Matrix::translate(offset);
            child.set_size(child_size);
        }
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut skia::Canvas) {
        if let Some(child) = state.child() {
            canvas.save();
            canvas.concat(&self.matrix);
            child.draw(canvas);
            canvas.restore();
        }
    }
}
