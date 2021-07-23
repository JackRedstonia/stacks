use super::ContainerSize;
use crate::prelude::*;

// Sets the behaviour of Contained when setting its child's size.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ContainMode {
    // Respect the child's layout sizing rule.
    // Contained<T> will call set_size on Wrap<T> with a Size respecting
    // the LayoutSize that Wrap::<T>::size() returned.
    Box,

    // Do nothing but inject the child's layout sizing rule reported to
    // Contained<T>'s parent. Will directly Wrap::<T>::set_size whatever it gets
    // from its own set_size.
    // Useful for CenterContainer (Contained<CenterContainer<T>>)
    Inject,
}

pub struct Contained<T: Widget> {
    inner: Wrap<T>,
    inner_size: LayoutSize,
    size: ContainerSize,
    mode: ContainMode,
}

impl<T: Widget> Contained<T> {
    pub fn new(
        inner: Wrap<T>,
        size: ContainerSize,
        mode: ContainMode,
    ) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            inner,
            inner_size: LayoutSize::default(),
            size,
            mode,
        }
        .into()
    }
}

impl<T: Widget> Widget for Contained<T> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        self.inner.load(stack);
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.inner.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        self.inner.input(event)
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let (inner_size, changed) = self.inner.size();
        self.inner_size = inner_size;
        (self.size.apply(&inner_size), changed)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        match self.mode {
            ContainMode::Box => {
                self.inner.set_size(self.inner_size.layout_one(size));
            }
            ContainMode::Inject => {
                self.inner.set_size(size);
            }
        }
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        self.inner.draw(canvas);
    }
}
