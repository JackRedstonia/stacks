pub mod widgets;

use super::game::{Error, Game, InputEvent, State};
use crate::skia::{Canvas, Color4f, Paint, Rect, Size};
use widgets::{LayoutSize, Widget, Wrap};

pub struct Framework<T: Widget> {
    root: Wrap<T>,
    layout_size: LayoutSize,
    size: Size,
}

impl<T: Widget> Framework<T> {
    pub fn new(root: Wrap<T>) -> Self {
        Self {
            root,
            layout_size: LayoutSize::ZERO,
            size: Size::new_empty(),
        }
    }
}

impl<T: Widget> Game for Framework<T> {
    fn update(&mut self) {
        let (size, changed) = self.root.size();
        if size != self.layout_size || changed {
            self.layout_size = size;
            self.root.set_size(self.size);
        }
        self.root.update();
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        self.root.draw(canvas);
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

    fn set_size(&mut self, window_size: Size) {
        self.size = Size::new(
            self.layout_size.width.min.max(window_size.width),
            self.layout_size.height.min.max(window_size.height),
        );
        self.root.set_size(self.size);
    }

    fn input(&mut self, event: InputEvent) {
        self.root.input(&event);
    }

    fn close(&mut self) {}

    fn crash(&mut self, err: Error) {
        println!("Stacks has crashed!\nMore info: {:?}", err);
    }
}
