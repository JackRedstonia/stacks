use crate::prelude::*;

pub struct FullscreenContainer {
    key: Keycode,
    lock: bool,
}

impl FullscreenContainer {
    pub fn new() -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            key: Keycode::F11,
            lock: false,
        }
        .into()
    }
}

impl Widget for FullscreenContainer {
    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        match event {
            InputEvent::KeyDown(k) if *k == self.key => {
                if !self.lock {
                    self.lock = true;
                    State::toggle_fullscreen();
                }
                true
            }
            InputEvent::KeyUp(k) if *k == self.key => {
                self.lock = false;
                true
            }
            _ => state.child().map(|e| e.input(event)).unwrap_or(false),
        }
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        state.child().map(|e| e.size()).unwrap_or_default()
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        if let Some(child) = state.child() {
            child.set_size(size);
        }
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut skia::Canvas) {
        if let Some(child) = state.child() {
            child.draw(canvas);
        }
    }
}
