pub mod resource;
pub mod widgets;

use std::cell::RefCell;

use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use crate::game::{
    Builder, Game, GameError, InputEvent, RunnerError, State, ID,
};
use crate::prelude::*;
use resource::ResourceStack;
use skia::{Canvas, Size};
use widgets::{LayoutSize, Widget, Wrap};

#[derive(Debug)]
pub enum FrameworkError {
    WidgetCreationError(Box<dyn StdError + 'static>),
    RunnerError(RunnerError),
}

impl Display for FrameworkError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            FrameworkError::WidgetCreationError(e) => {
                write!(f, "widget creation error: {}", e)
            }
            FrameworkError::RunnerError(e) => {
                write!(f, "runner error: {}", e)
            }
        }
    }
}

impl StdError for FrameworkError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            FrameworkError::WidgetCreationError(e) => Some(e.as_ref()),
            FrameworkError::RunnerError(e) => Some(e),
        }
    }
}

pub struct Framework<T: Widget + ?Sized> {
    root: Wrap<T>,
    layout_size: LayoutSize,
    size: Size,
    schedule_set_size: bool,

    recycled_resource_stack: ResourceStack,
}

impl<T: Widget + 'static> Framework<T> {
    pub fn run<F>(name: &str, root: F) -> FrameworkError
    where
        F: FnOnce() -> Result<Wrap<T>, Box<dyn StdError + 'static>>,
    {
        let e = Builder::new().window_title(name).run(|| {
            FrameworkState::init();
            let root =
                root().map_err(|e| FrameworkError::WidgetCreationError(e))?;
            Ok(Self::new(root))
        });
        match e {
            Ok(e) => e,
            Err(e) => FrameworkError::RunnerError(e),
        }
    }

    pub fn new(root: Wrap<T>) -> Self {
        Self {
            root,
            layout_size: LayoutSize::ZERO,
            size: Size::new_empty(),
            schedule_set_size: false,
            recycled_resource_stack: ResourceStack::new(),
        }
    }

    fn focus_aware_input(&mut self, event: InputEvent) {
        let remove_hover = FrameworkState::with_mut(|x| {
            if x.just_grabbed_focus {
                x.just_grabbed_focus = false;
                return x.current_focused_id;
            }
            None
        });
        if let Some(id) = remove_hover {
            self.root.input(&InputEvent::RemoveHoverExcept(id));
        }
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

impl<T: Widget + 'static> Game for Framework<T> {
    fn update(&mut self) {
        self.root.update();
        self.maybe_load();
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        // Trigger widget wrappers to check whether they are hovered on
        self.focus_aware_input(InputEvent::MouseMove(State::mouse_position()));

        // Trigger layout
        let (size, changed) = self.root.size();
        let resized = size != self.layout_size;
        if self.schedule_set_size || resized || changed {
            self.schedule_set_size = false;
            self.layout_size = size;
            self.root.set_size(self.layout_size.layout_one(self.size));
        }
        if resized {
            State::set_min_window_size(size.get_min());
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
        self.schedule_set_size = true;
        self.maybe_load();
    }

    fn input(&mut self, event: InputEvent) {
        self.focus_aware_input(event);
        self.maybe_load();
    }

    fn close(&mut self) {}

    fn crash(&mut self, err: GameError) {
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

    pub fn init() {
        FrameworkState::STATE
            .with(|x| *x.borrow_mut() = Some(Default::default()));
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
        Self::STATE
            .with(|x| f(x.borrow_mut().as_mut().expect(Self::PANIC_MESSAGE)))
    }
}
