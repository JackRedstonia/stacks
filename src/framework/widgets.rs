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
pub use text::{
    FontName, FontResource, FontStyle, Fonts, Text, TextLayoutMode,
};
pub use transform::Transform;
pub use wrap::{WidgetBorrow, WidgetBorrowMut, WidgetState, Wrap, Wrappable};

use crate::game::InputEvent;
use crate::skia::{Canvas, Size};

use super::resource::ResourceStack;

#[allow(unused_variables)]
pub trait Widget {
    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack);

    fn update(&mut self, state: &mut WidgetState);

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        false
    }

    fn hover(&mut self, state: &mut WidgetState) {}

    fn hover_lost(&mut self, state: &mut WidgetState) {}

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        (LayoutSize::ZERO, false)
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {}

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {}
}
