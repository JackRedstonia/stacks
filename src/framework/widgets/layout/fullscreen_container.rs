use crate::prelude::*;

pub struct FullscreenContainer<T: Widget> {
    inner: Wrap<T>,
    key: Keycode,
    lock: bool,
}

impl<T: Widget> FullscreenContainer<T> {
    pub fn new(inner: Wrap<T>) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            inner: inner.into(),
            key: Keycode::F11,
            lock: false,
        }
        .into()
    }
}

impl<T: Widget> Widget for FullscreenContainer<T> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.inner.load(stack);
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.inner.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
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
            _ => self.inner.input(event),
        }
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        self.inner.size()
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.inner.set_size(size);
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut skia::Canvas) {
        self.inner.draw(canvas);
    }
}
