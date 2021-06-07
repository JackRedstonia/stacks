use std::collections::HashSet;

use crate::skia::{scalar, Matrix, Point, Size};
use glutin::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use glutin::event::{
    ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
    WindowEvent,
};

use super::ID;

#[derive(Debug, PartialEq, Clone)]
pub enum InputEvent {
    KeyDown(VirtualKeyCode),
    KeyUp(VirtualKeyCode),
    MouseMove(Point),
    MouseDown(MouseButton, Point),
    MouseUp(MouseButton, Point),
    MouseScroll(ScrollAmount, Point),
    CharReceived(char),
    Focused(ID, Box<Self>),
    RemoveHoverExcept(ID),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ScrollAmount {
    Lines(Point),
    Pixels(Point),
}

impl InputEvent {
    pub fn position(&self) -> Option<Point> {
        Some(match self {
            Self::MouseMove(p) => *p,
            Self::MouseDown(_, p) => *p,
            Self::MouseUp(_, p) => *p,
            Self::MouseScroll(_, p) => *p,
            Self::Focused(_, e) => e.position()?,
            _ => return None,
        })
    }

    pub fn is_consumable(&self) -> bool {
        match self {
            InputEvent::MouseMove(_)
            | InputEvent::MouseUp(_, _)
            | InputEvent::KeyUp(_)
            | InputEvent::RemoveHoverExcept(_)
            | InputEvent::Focused(..) => false,
            InputEvent::KeyDown(_)
            | InputEvent::MouseDown(_, _)
            | InputEvent::MouseScroll(_, _)
            | InputEvent::CharReceived(_) => true,
        }
    }

    fn position_mut_ref(&mut self) -> Option<&mut Point> {
        Some(match self {
            Self::MouseMove(p) => p,
            Self::MouseDown(_, p) => p,
            Self::MouseUp(_, p) => p,
            Self::MouseScroll(_, p) => p,
            Self::Focused(_, e) => e.position_mut_ref()?,
            _ => return None,
        })
    }

    pub fn reverse_map_position(&self, matrix: Matrix) -> Option<Self> {
        let m = matrix.invert()?;
        let mut new_self = self.clone();
        if let Some(p) = new_self.position_mut_ref() {
            *p = m.map_point(*p);
        }
        Some(new_self)
    }
}

pub struct InputState {
    pub window_size: Size,
    scale_factor: f64,
    keys: HashSet<VirtualKeyCode>,
    mouse_position: Point,
    mouse_buttons: HashSet<MouseButton>,
}

pub enum EventHandleResult {
    Input(InputEvent),
    Resized(Size),
    Exit,
}

impl InputState {
    pub fn new(window_size: PhysicalSize<u32>, scale_factor: f64) -> Self {
        let LogicalSize::<f32> { width, height } =
            window_size.to_logical(scale_factor);
        Self {
            window_size: Size::new(width, height),
            scale_factor,
            keys: HashSet::new(),
            mouse_position: Point::default(),
            mouse_buttons: HashSet::new(),
        }
    }

    pub fn handle_event(
        &mut self,
        event: WindowEvent,
    ) -> Option<EventHandleResult> {
        match event {
            WindowEvent::CloseRequested => {
                return Some(EventHandleResult::Exit)
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => {
                self.scale_factor = scale_factor;
                let s = new_inner_size.to_logical(self.scale_factor);
                self.window_size = Size::new(s.width, s.height);
                return Some(EventHandleResult::Resized(self.window_size));
            }
            WindowEvent::Resized(phys_size) => {
                let s = phys_size.to_logical(self.scale_factor);
                self.window_size = Size::new(s.width, s.height);
                return Some(EventHandleResult::Resized(self.window_size));
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = state == ElementState::Pressed;
                let e = if is_pressed {
                    self.keys.insert(keycode);
                    InputEvent::KeyDown(keycode)
                } else {
                    self.keys.remove(&keycode);
                    InputEvent::KeyUp(keycode)
                };
                return Some(EventHandleResult::Input(e));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let is_pressed = state == ElementState::Pressed;
                let e = if is_pressed {
                    self.mouse_buttons.insert(button);
                    InputEvent::MouseDown(button, self.mouse_position)
                } else {
                    self.mouse_buttons.remove(&button);
                    InputEvent::MouseUp(button, self.mouse_position)
                };
                return Some(EventHandleResult::Input(e));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let e = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        ScrollAmount::Lines((x, y).into())
                    }
                    MouseScrollDelta::PixelDelta(a) => {
                        let a: LogicalPosition<scalar> =
                            a.to_logical(self.scale_factor);
                        ScrollAmount::Pixels((a.x, a.y).into())
                    }
                };
                return Some(EventHandleResult::Input(
                    InputEvent::MouseScroll(e, self.mouse_position),
                ));
            }
            WindowEvent::CursorMoved { position, .. } => {
                let position: LogicalPosition<scalar> =
                    position.to_logical(self.scale_factor);
                let position = Point::new(position.x, position.y);
                self.mouse_position = position;
                return Some(EventHandleResult::Input(InputEvent::MouseMove(
                    position,
                )));
            }
            WindowEvent::ReceivedCharacter(ch) => {
                if !ch.is_control() {
                    return Some(EventHandleResult::Input(
                        InputEvent::CharReceived(ch),
                    ));
                }
            }
            _ => {}
        }
        None
    }

    pub fn mouse_position(&self) -> Point {
        self.mouse_position
    }

    /// Returns whether the given key is down
    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.keys.contains(&key)
    }

    /// Returns whether the given button is down
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }
}
