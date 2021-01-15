pub mod music;
pub mod widgets;

use std::{cell::RefCell, collections::VecDeque};

use crate::prelude::*;
use game::{Error, Game, InputEvent, State, ID};
use skia::{Canvas, Color4f, Data, Image, Paint, Point, Size};
use widgets::{LayoutSize, Widget, Wrap};

const CURSOR: &[u8] = include_bytes!("resources/cursor.png");
const CURSORTRAIL: &[u8] = include_bytes!("resources/cursortrail.png");

pub struct Framework<T: Widget> {
    root: Wrap<T>,
    layout_size: LayoutSize,
    size: Size,

    cursor: Image,
    cursor_trail: Image,
    cursor_history: VecDeque<(Point, f32)>,

    cursor_scale: f32,
    cursor_fade_time: f32,
}

impl<T: Widget> Framework<T> {
    pub fn new(root: impl Into<Wrap<T>>) -> Self {
        let state = FrameworkState {
            current_focused_id: None,
            just_grabbed_focus: false,
        };
        FrameworkState::STATE.with(|x| *x.borrow_mut() = Some(state));
        Self {
            root: root.into(),
            layout_size: LayoutSize::ZERO,
            size: Size::new_empty(),

            cursor: Image::from_encoded(
                // SAFETY: `Data::new_bytes(ptr)` requires that `ptr` outlives
                // the Data struct, which lives for the rest of the program.
                // This is guaranteed by that `CURSOR` is `'static`.
                unsafe { Data::new_bytes(CURSOR) },
            )
            .unwrap(),
            cursor_trail: Image::from_encoded(
                // SAFETY: Same as above.
                unsafe { Data::new_bytes(CURSORTRAIL) },
            )
            .unwrap(),

            cursor_history: VecDeque::new(),
            cursor_scale: 0.5,
            cursor_fade_time: 0.1,
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
}

impl<T: Widget> Game for Framework<T> {
    fn update(&mut self) {
        self.root.update();
        let n = State::elapsed().as_secs_f32() - self.cursor_fade_time;
        while let Some((_, t)) = self.cursor_history.iter().next() {
            if *t < n {
                self.cursor_history.pop_front();
            } else {
                break;
            }
        }
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

        // Draw cursor & cursor trail
        let scale = self.cursor_scale;
        let scale_inv = 1.0 / scale;
        let mouse_pos = State::mouse_position() * scale_inv;
        let cursor_mid = self.cursor.dimensions().center();
        let trail_mid: Point = self.cursor_trail.dimensions().center();
        canvas.save();
        canvas.scale((scale, scale));
        let rel_zero = State::elapsed().as_secs_f32() - self.cursor_fade_time;
        for &(p, t) in &self.cursor_history {
            if t > rel_zero {
                let opacity = (t - rel_zero) / self.cursor_fade_time;
                let color = Color4f::new(1.0, 1.0, 1.0, opacity);
                canvas.draw_image(
                    &self.cursor_trail,
                    p * scale_inv - trail_mid,
                    Some(&Paint::new(color, None)),
                );
            }
        }
        canvas.draw_image(&self.cursor, mouse_pos - cursor_mid, None);
        canvas.restore();
    }

    fn set_size(&mut self, window_size: Size) {
        self.size = Size::new(
            self.layout_size.width.min.max(window_size.width),
            self.layout_size.height.min.max(window_size.height),
        );
        self.root.set_size(self.size);
    }

    fn input(&mut self, event: InputEvent) {
        if let InputEvent::MouseMove(pos) = &event {
            self.cursor_history
                .push_back((*pos, State::elapsed().as_secs_f32()))
        }
        self.focus_aware_input(event);
    }

    fn close(&mut self) {}

    fn crash(&mut self, err: Error) {
        println!("Stacks has crashed!\nMore info: {:?}", err);
    }
}

pub struct FrameworkState {
    current_focused_id: Option<ID>,
    just_grabbed_focus: bool,
}

impl FrameworkState {
    const PANIC_MESSAGE: &'static str =
        "Attempt to get framework state while framework is uninitialised";
    thread_local!(static STATE: RefCell<Option<FrameworkState>> = RefCell::new(None));

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
