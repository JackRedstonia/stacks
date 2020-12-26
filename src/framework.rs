pub mod widgets;

use super::game::{Error, Game, InputEvent, State};
use crate::skia::{Color4f, Paint, Rect, Size, Canvas};
use widgets::{Widget, Wrap};

pub struct Framework<T: Widget> {
    root: Wrap<T>,
}

impl<T: Widget> Framework<T> {
    pub fn new(root: Wrap<T>) -> Self {
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
        canvas.draw_rect(
            Rect::new(-5.0, -5.0, 5.0, 5.0).with_offset(State::with(|x| {
                (
                    x.input_state.mouse_position.x,
                    x.input_state.mouse_position.y,
                )
            })),
            &Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None),
        );
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
