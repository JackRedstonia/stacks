pub mod widgets;

use super::game::Canvas;
use super::game::{Error, Game, InputEvent, State};
use crate::skia::Size;
use widgets::Widget;

pub struct Framework<T: Widget> {
    root: T,
}

impl<T: Widget> Framework<T> {
    pub fn new(root: T) -> Self {
        Self { root }
    }
}

impl<T: Widget> Game for Framework<T> {
    fn update(&mut self) {
        self.root.update();
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        let size = self.root.size().layout_one(State::with(|x| {
            let win = x.input_state.window_size;
            Size::new(win.width, win.height)
        }));
        self.root.draw(canvas, size);
    }

    fn input(&mut self, event: InputEvent) {
        let size = State::with(|x| {
            let win = x.input_state.window_size;
            Size::new(win.width, win.height)
        });
        self.root.input(&event, size);
    }

    fn close(&mut self) {}

    fn crash(&mut self, err: Error) {
        println!("Stacks has crashed!\nMore info: {:?}", err);
    }
}
