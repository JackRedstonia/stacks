use crate::prelude::*;

pub struct Backgrounded;

impl Backgrounded {
    pub fn new() -> Wrap<Self> {
        Self.into()
    }
}

impl Widget for Backgrounded {
    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        state
            .children()
            .skip(1)
            .next()
            .map(|e| e.input(event))
            .unwrap_or(false)
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        state
            .children()
            .skip(1)
            .next()
            .map(|e| e.size())
            .unwrap_or_default()
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        for i in state.children().take(2) {
            i.set_size(size);
        }
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        for i in state.children().take(2) {
            i.draw(canvas);
        }
    }
}
