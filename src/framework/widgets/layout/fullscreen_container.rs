use crate::prelude::*;

pub struct FullscreenContainer<T: Widget + ?Sized> {
    child: Wrap<T>,
    key: Keycode,
    lock: bool,
}

impl<T: Widget + ?Sized> FullscreenContainer<T> {
    pub fn new(child: Wrap<T>) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            child,
            key: Keycode::F11,
            lock: false,
        }
        .into()
    }
}

impl<T: Widget + ?Sized> Widget for FullscreenContainer<T> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.child.load(stack);
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.child.update();
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
            _ => self.child.input(event),
        }
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        self.child.size()
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.child.set_size(size);
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut skia::Canvas) {
        self.child.draw(canvas);
    }
}
