//! Handles input tracking and provides an easy way to detect clicks, dragging, etc.

use skulpin::app::VirtualKeyCode;
use skulpin::winit;
// Re-export winit types
pub use winit::dpi::LogicalPosition;
pub use winit::dpi::LogicalSize;
pub use winit::dpi::PhysicalPosition;
pub use winit::dpi::PhysicalSize;
pub use winit::dpi::Position;
pub use winit::dpi::Size;
pub use winit::event::ElementState;
pub use winit::event::MouseButton;
pub use winit::event::MouseScrollDelta;

use crate::scalar;
use crate::winit::window::Window;

use super::AppControl;

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
    pub fn reverse_map_position(&self, matrix: &skulpin::skia_safe::Matrix) -> Option<Self> {
        let m = matrix.invert()?;
        Some(match self {
            Self::MouseMove(LogicalPosition { x, y }) => {
                let skulpin::skia_safe::Point { x, y } = m.map_point((*x, *y));
                Self::MouseMove(LogicalPosition { x, y })
            },
            Self::MouseDown(b, LogicalPosition { x, y }) => {
                let skulpin::skia_safe::Point { x, y } = m.map_point((*x, *y));
                Self::MouseDown(b.clone(), LogicalPosition { x, y })
            },
            Self::MouseUp(b, LogicalPosition { x, y }) => {
                let skulpin::skia_safe::Point { x, y } = m.map_point((*x, *y));
                Self::MouseUp(b.clone(), LogicalPosition { x, y })
            },
            e => e.clone(),
        })
    }
}

/// State of input devices. This is maintained by processing events from winit
pub struct InputState {
    pub window_size: PhysicalSize<u32>,
    pub scale_factor: f64,
    pub events: Vec<InputEvent>,
    pub key_is_down: [bool; Self::KEYBOARD_BUTTON_COUNT],
    pub mouse_position: LogicalPosition<scalar>,
    pub mouse_button_is_down: [bool; Self::MOUSE_BUTTON_COUNT],
    pub mouse_wheel_delta: MouseScrollDelta,
}

impl InputState {
    /// Number of keyboard buttons we will track. Any button with a higher virtual key code will be
    /// ignored
    pub const KEYBOARD_BUTTON_COUNT: usize = 255;

    /// Number of mouse buttons we will track. Any button with a higher index will be ignored.
    pub const MOUSE_BUTTON_COUNT: usize = 7;

    /// Create a new input state to track the given window
    pub fn new(window: &Window) -> InputState {
        InputState {
            window_size: window.inner_size(),
            scale_factor: window.scale_factor(),
            events: vec![],
            key_is_down: [false; Self::KEYBOARD_BUTTON_COUNT],
            mouse_position: LogicalPosition::new(0.0, 0.0),
            mouse_button_is_down: [false; Self::MOUSE_BUTTON_COUNT],
            mouse_wheel_delta: MouseScrollDelta::LineDelta(0.0, 0.0),
        }
    }

    /// Call when winit sends an event
    pub fn handle_winit_event<T>(
        &mut self,
        app_control: &mut AppControl,
        event: &winit::event::Event<T>,
        _window_target: &winit::event_loop::EventLoopWindowTarget<T>,
    ) {
        use crate::winit::event::Event;
        use crate::winit::event::WindowEvent;

        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => app_control.enqueue_terminate_process(),
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                } => {
                    self.window_size = **new_inner_size;
                    self.scale_factor = *scale_factor;
                }
                WindowEvent::Resized(window_size) => self.window_size = *window_size,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(vk) = input.virtual_keycode {
                        if let Some(kc) = Self::keyboard_button_to_index(vk) {
                            let v = input.state == ElementState::Pressed;
                            self.key_is_down[kc] = v;
                            self.events.push(if v {
                                InputEvent::KeyDown(vk)
                            } else {
                                InputEvent::KeyUp(vk)
                            });
                        };
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if let Some(button_index) = Self::mouse_button_to_index(*button) {
                        let v = matches!(state, ElementState::Pressed);
                        self.mouse_button_is_down[button_index] = v;
                        self.events.push(if v {
                            InputEvent::MouseDown(*button, self.mouse_position)
                        } else {
                            InputEvent::MouseUp(*button, self.mouse_position)
                        });
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let logical = position.to_logical(self.scale_factor);
                    self.mouse_position = logical;
                    self.events
                        .push(InputEvent::MouseMove(logical));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    self.handle_mouse_wheel_event(delta);
                    self.events
                        .push(InputEvent::MouseScroll(*delta));
                }
                _ => {}
            }
        }
    }

    /// Returns true if the given key is down
    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        if let Some(index) = Self::keyboard_button_to_index(key) {
            self.key_is_down[index]
        } else {
            false
        }
    }

    /// Returns true if the given button is down
    pub fn is_mouse_down(&self, mouse_button: MouseButton) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_is_down[index]
        } else {
            false
        }
    }

    fn handle_mouse_wheel_event(&mut self, delta: &MouseScrollDelta) {
        // Try to add the delta to self.mouse_wheel_delta
        if let MouseScrollDelta::LineDelta(x1, y1) = self.mouse_wheel_delta {
            if let MouseScrollDelta::LineDelta(x2, y2) = delta {
                self.mouse_wheel_delta = MouseScrollDelta::LineDelta(x1 + *x2, y1 + *y2);
            } else {
                self.mouse_wheel_delta = *delta;
            }
        } else if let MouseScrollDelta::PixelDelta(d1) = self.mouse_wheel_delta {
            if let MouseScrollDelta::PixelDelta(d2) = delta {
                self.mouse_wheel_delta = MouseScrollDelta::PixelDelta(PhysicalPosition::<f64>::new(
                    d1.x + d2.x,
                    d1.y + d2.y,
                ));
            } else {
                self.mouse_wheel_delta = *delta;
            }
        }

        self.mouse_wheel_delta = *delta;
    }

    /// Call at the end of every frame. This clears events that were "just" completed.
    pub fn end_frame(&mut self) {
        self.mouse_wheel_delta = MouseScrollDelta::LineDelta(0.0, 0.0);
        self.events.clear();
    }

    //
    // Helper functions
    //

    /// Convert the winit mouse button enum into a numerical index
    pub fn mouse_button_to_index(button: MouseButton) -> Option<usize> {
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
    pub fn keyboard_button_to_index(button: VirtualKeyCode) -> Option<usize> {
        let index = button as usize;
        if index >= Self::KEYBOARD_BUTTON_COUNT {
            None
        } else {
            Some(index)
        }
    }
}
