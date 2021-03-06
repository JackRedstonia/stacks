use crate::prelude::*;

pub struct FullscreenContainer<T: Widget> {
    child: Wrap<T>,
    key: Keycode,
    lock: bool,
}

impl<T: Widget> FullscreenContainer<T> {
    pub fn new(child: Wrap<T>) -> Wrap<Self> {
        Self {
            child,
            key: Keycode::F11,
            lock: false,
        }
        .into()
    }
}

impl<T: Widget> Widget for FullscreenContainer<T> {
    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack) {
        self.child.load(stack);
    }

    fn update(&mut self, state: &mut WidgetState) {
        self.child.update();
    }

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
            _ => self.child.input(event),
        }
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        self.child.size()
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.child.set_size(size);
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut skia::Canvas) {
        self.child.draw(canvas);
    }
}
