use crate::prelude::*;

pub struct SizeFillContainer {
    size: Size,
    child_size: LayoutSize,
    target_size: Size,
    matrix: Matrix,
}

impl SizeFillContainer {
    pub fn new(target_size: Size) -> Wrap<Self> {
        Self {
            size: Size::default(),
            child_size: LayoutSize::ZERO,
            target_size,
            matrix: Matrix::default(),
        }
        .into()
    }
}

impl Widget for SizeFillContainer {
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
        (
            LayoutSize::ZERO.expand_width().expand_height(),
            state
                .child()
                .map(|child| {
                    let (child_size, changed) = child.size();
                    self.child_size = child_size;
                    changed
                })
                .unwrap_or(false),
        )
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.size = size;
        let scale = (size.width / self.target_size.width)
            .min(size.height / self.target_size.height);
        self.matrix = Matrix::scale((scale, scale));
        if let Some(child) = state.child() {
            let child_max_size = size / scale;
            child.set_size(self.child_size.layout_one(child_max_size));
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
