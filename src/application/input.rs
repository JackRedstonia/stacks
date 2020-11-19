use skia_safe::{scalar, Point, Matrix};
use skulpin_renderer::skia_safe;
use skulpin_renderer_winit::winit;
use winit::{
    dpi::LogicalPosition,
    dpi::LogicalSize,
    event::{
        ElementState, Event as WinitEvent, MouseButton, MouseScrollDelta, VirtualKeyCode,
        WindowEvent,
    },
    window::Window,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputEvent {
    KeyDown(VirtualKeyCode),
    KeyUp(VirtualKeyCode),
    MouseMove(LogicalPosition<scalar>),
    MouseDown(MouseButton, LogicalPosition<scalar>),
    MouseUp(MouseButton, LogicalPosition<scalar>),
    MouseScroll(MouseScrollDelta),
}

impl InputEvent {
    pub fn reverse_map_position(&self, matrix: Matrix) -> Option<Self> {
        let m = matrix.invert()?;
        Some(match self {
            Self::MouseMove(LogicalPosition { x, y }) => {
                let Point { x, y } = m.map_point((*x, *y));
                Self::MouseMove(LogicalPosition { x, y })
            }
            Self::MouseDown(b, LogicalPosition { x, y }) => {
                let Point { x, y } = m.map_point((*x, *y));
                Self::MouseDown(b.clone(), LogicalPosition { x, y })
            }
            Self::MouseUp(b, LogicalPosition { x, y }) => {
                let Point { x, y } = m.map_point((*x, *y));
                Self::MouseUp(b.clone(), LogicalPosition { x, y })
            }
            e => e.clone(),
        })
    }
}

pub struct InputState {
    pub window_size: LogicalSize<scalar>,
    pub scale_factor: f64,

    pub keys: [bool; Self::KEYBOARD_BUTTON_COUNT],
    pub mouse_position: LogicalPosition<scalar>,
    pub mouse_buttons: [bool; Self::MOUSE_BUTTON_COUNT],
}

pub enum EventHandleResult {
    Input(InputEvent),
    Exit,
}

impl InputState {
    pub const KEYBOARD_BUTTON_COUNT: usize = 255;
    pub const MOUSE_BUTTON_COUNT: usize = 7;

    pub fn new(window: &Window) -> Self {
        let scale_factor = window.scale_factor();
        Self {
            window_size: window.inner_size().to_logical(scale_factor),
            scale_factor,
            keys: [false; Self::KEYBOARD_BUTTON_COUNT],
            mouse_position: LogicalPosition::new(0.0, 0.0),
            mouse_buttons: [false; Self::MOUSE_BUTTON_COUNT],
        }
    }

    pub fn handle_event<T>(&mut self, event: &WinitEvent<T>) -> Option<EventHandleResult> {
        if let WinitEvent::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => return Some(EventHandleResult::Exit),
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                    ..
                } => {
                    self.scale_factor = *scale_factor;
                    self.window_size = new_inner_size.to_logical(*scale_factor);
                }
                WindowEvent::Resized(window_size) => {
                    self.window_size = window_size.to_logical(self.scale_factor)
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(k) = input.virtual_keycode {
                        if let Some(kc) = Self::keyboard_button_to_index(k) {
                            let pressed = input.state == ElementState::Pressed;
                            self.keys[kc] = pressed;
                            return Some(EventHandleResult::Input(if pressed {
                                InputEvent::KeyDown(k)
                            } else {
                                InputEvent::KeyUp(k)
                            }));
                        }
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if let Some(idx) = Self::mouse_button_to_index(*button) {
                        let pressed = *state == ElementState::Pressed;
                        self.mouse_buttons[idx] = pressed;
                        return Some(EventHandleResult::Input(if pressed {
                            InputEvent::MouseDown(*button, self.mouse_position)
                        } else {
                            InputEvent::MouseUp(*button, self.mouse_position)
                        }));
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let logical = position.to_logical(self.scale_factor);
                    self.mouse_position = logical;
                    return Some(EventHandleResult::Input(InputEvent::MouseMove(logical)));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    return Some(EventHandleResult::Input(InputEvent::MouseScroll(*delta)));
                }
                _ => {}
            }
        }
        None
    }

    /// Returns true if the given key is down
    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        if let Some(index) = Self::keyboard_button_to_index(key) {
            self.keys[index]
        } else {
            false
        }
    }

    /// Returns true if the given button is down
    pub fn is_mouse_down(&self, mouse_button: MouseButton) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_buttons[index]
        } else {
            false
        }
    }

    /// Convert the winit mouse button enum into a numerical index
    fn mouse_button_to_index(button: MouseButton) -> Option<usize> {
        let index = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Other(x) => (x as usize) + 3,
        };

        if index >= Self::MOUSE_BUTTON_COUNT {
            None
        } else {
            Some(index)
        }
    }

    /// Convert the winit virtual key code into a numerical index
    fn keyboard_button_to_index(button: VirtualKeyCode) -> Option<usize> {
        let index = button as usize;
        if index >= Self::KEYBOARD_BUTTON_COUNT {
            None
        } else {
            Some(index)
        }
    }
}
