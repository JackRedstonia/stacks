pub mod components;

use super::game::Canvas;
use super::game::{Error, Game, InputEvent, InputState, TimeState};
use crate::skia::Size;
use components::{Component, LayoutDimension, LayoutSize};

pub struct Framework<T: Component> {
    root: T,
}

impl<T: Component> Framework<T> {
    pub fn new(root: T) -> Self {
        Self { root }
    }
}

impl<T: Component> Game for Framework<T> {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState) {
        self.root.update(input_state, time_state);
    }

    fn draw(&mut self, input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas) {
        let _ = self.root.size(input_state, time_state);
        self.root.draw(
            input_state,
            time_state,
            canvas,
            Size::new(
                input_state.window_size.width,
                input_state.window_size.height,
            ),
        );
    }

    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: InputEvent) {
        self.root.input(
            input_state,
            time_state,
            &event,
            Size::new(
                input_state.window_size.width,
                input_state.window_size.height,
            ),
        );
    }

    fn close(&mut self) {}

    fn crash(&mut self, err: Error) {
        println!("Stacks has crashed!\nMore info: {:?}", err);
    }
}
