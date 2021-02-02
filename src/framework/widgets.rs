pub mod audio;
pub mod layout;
pub mod shapes;
pub mod ui;

mod backgrounded;
mod layout_size;
mod text;
mod transform;
mod wrap;

pub use backgrounded::Backgrounded;
pub use layout_size::{LayoutDimension, LayoutSize};
pub use text::{Font, FontStyle, Text};
pub use transform::Transform;
pub use wrap::{WidgetBorrow, WidgetState, Wrap, Wrappable};

use crate::game::InputEvent;
use crate::skia::{Canvas, Size};

use super::resource::ResourceStack;

#[allow(unused_variables)]
pub trait Widget {
    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack) {
        for i in state.children() {
            i.load(stack);
        }
    }

    fn update(&mut self, state: &mut WidgetState) {
        for i in state.children() {
            i.update();
        }
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        for i in state.children().rev() {
            if i.input(event) {
                return true;
            }
        }
        false
    }

    fn hover(&mut self, state: &mut WidgetState) {}

    fn hover_lost(&mut self, state: &mut WidgetState) {}

    fn on_child_add(&mut self, child: &mut Wrap<dyn Widget>) {}

    fn on_child_remove(&mut self, child: &mut Wrap<dyn Widget>) {}

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        (LayoutSize::ZERO, false)
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {}

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {}
}
