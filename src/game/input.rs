use std::collections::HashSet;

use crate::skia;
use skia::{Matrix, Point, Size};
use skulpin_renderer::LogicalSize;
use skulpin_renderer_sdl2::sdl2::{
    event::{Event as Sdl2Event, WindowEvent},
    keyboard::Keycode,
    mouse::MouseButton,
};

use super::ID;

#[derive(Debug, PartialEq, Clone)]
pub enum InputEvent {
    KeyDown(Keycode),
    KeyUp(Keycode),
    MouseMove(Point),
    MouseDown(MouseButton, Point),
    MouseUp(MouseButton, Point),
    MouseScroll(i32),
    Focused(ID, Box<Self>),
    RemoveHoverExcept(ID),
}

impl InputEvent {
    pub fn position(&self) -> Option<Point> {
        Some(match self {
            Self::MouseMove(p) => *p,
            Self::MouseDown(_, p) => *p,
            Self::MouseUp(_, p) => *p,
            _ => return None,
        })
    }

    pub fn consumable(&self) -> bool {
        match self {
            InputEvent::MouseMove(_)
            | InputEvent::RemoveHoverExcept(_)
            | InputEvent::Focused(..) => false,
            InputEvent::KeyDown(_)
            | InputEvent::KeyUp(_)
            | InputEvent::MouseDown(_, _)
            | InputEvent::MouseUp(_, _)
            | InputEvent::MouseScroll(_) => true,
        }
    }

    pub fn reverse_map_position(&self, matrix: Matrix) -> Option<Self> {
        let m = matrix.invert()?;
        Some(match self {
            Self::MouseMove(p) => Self::MouseMove(m.map_point(*p)),
            Self::MouseDown(b, p) => Self::MouseDown(*b, m.map_point(*p)),
            Self::MouseUp(b, p) => Self::MouseUp(*b, m.map_point(*p)),
            Self::Focused(i, b) => {
                let inner = b.reverse_map_position(matrix)?;
                Self::Focused(*i, Box::new(inner))
            }
            _ => self.clone(),
        })
    }
}

pub struct InputState {
    pub window_size: Size,
    keys: HashSet<Keycode>,
    mouse_position: Point,
    mouse_buttons: HashSet<MouseButton>,
}

pub enum EventHandleResult {
    Input(InputEvent),
    Resized(Size),
    Exit,
}

impl InputState {
    pub const KEYBOARD_BUTTON_COUNT: usize = 255;
    pub const MOUSE_BUTTON_COUNT: usize = 5;

    pub fn new(window_size: LogicalSize) -> Self {
        Self {
            window_size: Size::new(
                window_size.width as _,
                window_size.height as _,
            ),
            keys: HashSet::new(),
            mouse_position: Point::default(),
            mouse_buttons: HashSet::new(),
        }
    }

    pub fn handle_event(
        &mut self,
        event: &Sdl2Event,
    ) -> Option<EventHandleResult> {
        match event {
            Sdl2Event::Quit { .. } => return Some(EventHandleResult::Exit),
            Sdl2Event::Window { win_event, .. } => match win_event {
                WindowEvent::Close => return Some(EventHandleResult::Exit),
                WindowEvent::Resized(width, height) => {
                    return Some(EventHandleResult::Resized(Size::new(
                        *width as _,
                        *height as _,
                    )));
                }
                _ => {}
            },
            Sdl2Event::KeyDown {
                keycode: Some(k), ..
            } => {
                self.keys.insert(*k);
                return Some(EventHandleResult::Input(InputEvent::KeyDown(*k)));
            }
            Sdl2Event::KeyUp {
                keycode: Some(k), ..
            } => {
                self.keys.remove(k);
                return Some(EventHandleResult::Input(InputEvent::KeyUp(*k)));
            }
            Sdl2Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                self.mouse_buttons.insert(*mouse_btn);
                let p = Point::new(*x as _, *y as _);
                return Some(EventHandleResult::Input(InputEvent::MouseDown(
                    *mouse_btn, p,
                )));
            }
            Sdl2Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                self.mouse_buttons.remove(mouse_btn);
                let p = Point::new(*x as _, *y as _);
                return Some(EventHandleResult::Input(InputEvent::MouseUp(
                    *mouse_btn, p,
                )));
            }
            Sdl2Event::MouseMotion { x, y, .. } => {
                let p = Point::new(*x as _, *y as _);
                self.mouse_position = p;
                return Some(EventHandleResult::Input(InputEvent::MouseMove(
                    p,
                )));
            }
            Sdl2Event::MouseWheel { y, .. } => {
                return Some(EventHandleResult::Input(
                    InputEvent::MouseScroll(*y),
                ));
            }
            _ => {}
        };
        None
    }

    pub fn mouse_position(&self) -> Point {
        self.mouse_position
    }

    /// Returns whether the given key is down
    pub fn is_key_down(&self, key: &Keycode) -> bool {
        self.keys.contains(&key)
    }

    /// Returns whether the given button is down
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }
}
