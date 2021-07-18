use crate::prelude::*;

use super::container::{ChildState, ContainerSize};

pub struct VContainer<T: Widget + ?Sized> {
    children: Vec<(Wrap<T>, ChildState)>,
    size: ContainerSize,
    spacing: scalar,
    sizes_changed: bool,
}

pub type VContainerDyn = VContainer<dyn Widget>;

impl<T: Widget + ?Sized> VContainer<T> {
    pub fn new(size: ContainerSize, spacing: Option<scalar>) -> Wrap<Self> {
        // `FrameworkState::request_load();` here is not needed, as there are
        // no children just yet.
        Self {
            children: vec![],
            size,
            spacing: spacing.unwrap_or(0.0),
            sizes_changed: false,
        }
        .into()
    }

    pub fn add_child(&mut self, child: Wrap<T>) -> &mut Self {
        FrameworkState::request_load();
        self.children.push((child, ChildState::new()));
        self
    }

    fn layout(&mut self, _state: &mut WidgetState, size: Size) {
        let total_space = size.height;

        let mut min = self.preoccupied();
        let mut expand = 0.0f32;

        for (_, state) in &self.children {
            min += state.layout_size.height.min;
            if let Some(e) = state.layout_size.height.expand {
                expand += e;
            }
        }

        let space_left = (total_space - min).max(0.0);
        let mut offset = 0.0;
        for (child, state) in &mut self.children {
            let mut height = state.layout_size.height.min;
            if let Some(e) = state.layout_size.height.expand {
                height += space_left * e / expand;
            }
            state.position.set(0.0, offset);
            offset += height + self.spacing;
            let width = if state.layout_size.width.expand.is_some() {
                size.width
            } else {
                state.layout_size.width.min.min(size.width)
            };
            state.maybe_set_size(child, Size::new(width, height));
        }
    }

    fn preoccupied(&self) -> scalar {
        (self.children.len() as scalar - 1.0).max(0.0) * self.spacing
    }
}

impl<T: Widget + ?Sized> Widget for VContainer<T> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        for (child, _) in &mut self.children {
            child.load(stack);
        }
    }

    fn update(&mut self, _state: &mut WidgetState) {
        for (child, _) in &mut self.children {
            child.update();
        }
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        let c = event.is_consumable();
        let mut any = false;
        for (child, state) in self.children.iter_mut().rev() {
            if let Some(event) =
                event.reverse_map_position(Matrix::translate(state.position))
            {
                if child.input(&event) {
                    any = true;
                    if c {
                        break;
                    }
                }
            }
        }
        any
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let mut height_min = self.preoccupied();
        let mut width_min = 0.0f32;

        self.sizes_changed = false;
        let mut children_changed = false;

        for (child, state) in &mut self.children {
            let (size, s, c) = state.size(child);
            self.sizes_changed |= s;
            children_changed |= c;
            height_min += size.height.min;
            width_min = width_min.max(size.width.min);
        }

        (
            self.size.apply(&LayoutSize::min(width_min, height_min)),
            children_changed,
        )
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.layout(state, size);
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        for (child, state) in &mut self.children {
            let m = Matrix::translate(state.position);
            canvas.save();
            canvas.concat(&m);
            child.draw(canvas);
            canvas.restore();
        }
    }
}
