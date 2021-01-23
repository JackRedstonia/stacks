pub mod resource;
pub mod widgets;

use std::cell::RefCell;

use crate::prelude::*;
use game::{Error, Game, InputEvent, State, ID};
use resource::ResourceStack;
use skia::{Canvas, Size};
use widgets::{LayoutSize, Widget, Wrap};

pub struct Framework<T: Widget> {
    root: Wrap<T>,
    layout_size: LayoutSize,
    size: Size,

    recycled_resource_stack: ResourceStack,
}

impl<T: Widget> Framework<T> {
    pub fn new(root: impl Into<Wrap<T>>) -> Self {
        Self {
            root: root.into(),
            layout_size: LayoutSize::ZERO,
            size: Size::new_empty(),
            recycled_resource_stack: ResourceStack::new(),
        }
    }

    fn focus_aware_input(&mut self, event: InputEvent) {
        FrameworkState::with_mut(|x| {
            if x.just_grabbed_focus {
                x.just_grabbed_focus = false;
                let id = x.current_focused_id.expect("Framework state's current focused widget ID is None despite focus just being grabbed");
                self.root.input(&InputEvent::RemoveHoverExcept(id));
            }
        });
        if let Some(id) = FrameworkState::current_focus() {
            self.root.input(&InputEvent::Focused(id, Box::new(event)));
        } else {
            self.root.input(&event);
        }
    }

    fn maybe_load(&mut self) {
        if FrameworkState::consume_load_request() {
            self.root.load(&mut self.recycled_resource_stack);
            assert!(self.recycled_resource_stack.is_empty());
        }
    }
}

impl<T: Widget> Game for Framework<T> {
    fn update(&mut self) {
        self.root.update();
        self.maybe_load();
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        // Trigger widget wrappers to check whether they are hovered on
        self.focus_aware_input(InputEvent::MouseMove(State::mouse_position()));

        // Trigger layout
        let (size, changed) = self.root.size();
        if size != self.layout_size || changed {
            self.layout_size = size;
            self.root.set_size(self.size);
        }

        // Do the actual drawing
        self.root.draw(canvas);
        self.maybe_load();
    }

    fn set_size(&mut self, window_size: Size) {
        self.size = Size::new(
            self.layout_size.width.min.max(window_size.width),
            self.layout_size.height.min.max(window_size.height),
        );
        self.root.set_size(self.size);
        self.maybe_load();
    }

    fn input(&mut self, event: InputEvent) {
        self.focus_aware_input(event);
        self.maybe_load();
    }

    fn close(&mut self) {}

    fn crash(&mut self, err: Error) {
        println!("Stacks has crashed!\nMore info: {:?}", err);
    }
}

#[derive(Default)]
pub struct FrameworkState {
    current_focused_id: Option<ID>,
    just_grabbed_focus: bool,
    load_requested: bool,
}

impl FrameworkState {
    const PANIC_MESSAGE: &'static str =
        "Attempt to get framework state while framework is uninitialized";
    thread_local!(static STATE: RefCell<Option<FrameworkState>> = RefCell::new(None));

    pub fn initialize() {
        FrameworkState::STATE.with(|x| *x.borrow_mut() = Some(Default::default()));
    }

    pub fn request_load() {
        Self::with_mut(|x| x.load_requested = true);
    }

    fn consume_load_request() -> bool {
        Self::with_mut(|x| {
            let b = x.load_requested;
            x.load_requested = false;
            b
        })
    }

    pub fn current_focus() -> Option<ID> {
        Self::with(|x| x.current_focused_id)
    }

    pub fn grab_focus(id: ID) {
        Self::with_mut(|x| {
            x.current_focused_id = Some(id);
            x.just_grabbed_focus = true;
        });
    }

    pub fn release_focus(id: ID) {
        Self::with_mut(|x| {
            if let Some(prev) = x.current_focused_id {
                if prev == id {
                    x.current_focused_id = None;
                    x.just_grabbed_focus = false;
                }
            }
        });
    }

    pub fn force_release_focus() {
        Self::with_mut(|x| {
            x.current_focused_id = None;
            x.just_grabbed_focus = false;
        });
    }

    #[inline]
    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        Self::STATE.with(|x| f(x.borrow().as_ref().expect(Self::PANIC_MESSAGE)))
    }

    #[inline]
    fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        Self::STATE.with(|x| f(x.borrow_mut().as_mut().expect(Self::PANIC_MESSAGE)))
    }
}
