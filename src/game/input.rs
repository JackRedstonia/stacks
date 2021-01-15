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
    pub keys: [bool; Self::KEYBOARD_BUTTON_COUNT],
    pub mouse_position: Point,
    pub mouse_buttons: [bool; Self::MOUSE_BUTTON_COUNT],
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
            window_size: Size::new(window_size.width as _, window_size.height as _),
            keys: [false; Self::KEYBOARD_BUTTON_COUNT],
            mouse_position: Point::default(),
            mouse_buttons: [false; Self::MOUSE_BUTTON_COUNT],
        }
    }

    pub fn handle_event(&mut self, event: &Sdl2Event) -> Option<EventHandleResult> {
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
                if let Some(kc) = Self::keyboard_key_to_index(*k) {
                    self.keys[kc] = true;
                    return Some(EventHandleResult::Input(InputEvent::KeyDown(*k)));
                }
            }
            Sdl2Event::KeyUp {
                keycode: Some(k), ..
            } => {
                if let Some(kc) = Self::keyboard_key_to_index(*k) {
                    self.keys[kc] = false;
                    return Some(EventHandleResult::Input(InputEvent::KeyUp(*k)));
                }
            }
            Sdl2Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                if let Some(kc) = Self::mouse_button_to_index(*mouse_btn) {
                    self.mouse_buttons[kc] = true;
                    let p = Point::new(*x as _, *y as _);
                    return Some(EventHandleResult::Input(InputEvent::MouseDown(
                        *mouse_btn, p,
                    )));
                }
            }
            Sdl2Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                if let Some(kc) = Self::mouse_button_to_index(*mouse_btn) {
                    self.mouse_buttons[kc] = false;
                    let p = Point::new(*x as _, *y as _);
                    return Some(EventHandleResult::Input(InputEvent::MouseUp(*mouse_btn, p)));
                }
            }
            Sdl2Event::MouseMotion { x, y, .. } => {
                let p = Point::new(*x as _, *y as _);
                self.mouse_position = p;
                return Some(EventHandleResult::Input(InputEvent::MouseMove(p)));
            }
            Sdl2Event::MouseWheel { y, .. } => {
                return Some(EventHandleResult::Input(InputEvent::MouseScroll(*y)));
            }
            _ => {}
        };
        None
    }

    /// Returns whether the given key is down
    pub fn is_key_down(&self, key: Keycode) -> bool {
        Self::keyboard_key_to_index(key)
            .map(|k| self.keys[k])
            .unwrap_or(false)
    }

    /// Returns whether the given button is down
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        Self::mouse_button_to_index(button)
            .map(|k| self.mouse_buttons[k])
            .unwrap_or(false)
    }

    /// Convert the mouse button enum into a numerical index
    fn mouse_button_to_index(button: MouseButton) -> Option<usize> {
        let index = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::X1 => 3,
            MouseButton::X2 => 4,
            _ => return None,
        };

        if index >= Self::MOUSE_BUTTON_COUNT {
            None
        } else {
            Some(index)
        }
    }

    /// Convert the key code into a numerical index
    fn keyboard_key_to_index(key: Keycode) -> Option<usize> {
        let index = key as usize;
        if index >= Self::KEYBOARD_BUTTON_COUNT {
            None
        } else {
            Some(index)
        }
    }
}
