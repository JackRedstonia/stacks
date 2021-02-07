use crate::game::ID;
use crate::prelude::*;

use super::container::{ContainerSize, ContainerState};

use std::collections::HashMap;

pub struct VContainer {
    size: ContainerSize,
    spacing: scalar,
    states: HashMap<ID, ContainerState>,
    sizes_changed: bool,
}

impl VContainer {
    pub fn new(size: ContainerSize, spacing: Option<scalar>) -> Wrap<Self> {
        Self {
            size,
            spacing: spacing.unwrap_or(0.0),
            states: HashMap::new(),
            sizes_changed: false,
        }
        .into()
    }

    fn layout(&mut self, state: &mut WidgetState, size: Size) {
        let total_space = size.height;

        let mut min =
            (state.children().len() as scalar - 1.0).max(0.0) * self.spacing;
        let mut expand = 0.0f32;

        for child in state.children() {
            let state = self.states.get(&child.id()).unwrap();
            min += state.layout_size.height.min;
            if let Some(e) = state.layout_size.height.expand {
                expand += e;
            }
        }

        let space_left = (total_space - min).max(0.0);
        let mut offset = 0.0;
        for child in state.children() {
            let state = self.states.get_mut(&child.id()).unwrap();
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
}

impl Widget for VContainer {
    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        let c = event.consumable();
        let mut any = false;
        for i in state.children().rev() {
            let state = self.states.get(&i.id()).unwrap();
            if let Some(event) =
                event.reverse_map_position(Matrix::translate(state.position))
            {
                if i.input(&event) {
                    any = true;
                    if c {
                        break;
                    }
                }
            }
        }
        any
    }

    fn on_child_add(&mut self, child: &mut Wrap<dyn Widget>) {
        self.states.insert(child.id(), ContainerState::new());
    }

    fn on_child_remove(&mut self, child: &mut Wrap<dyn Widget>) {
        self.states.remove(&child.id());
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        let mut height_min = 0.0f32;
        let mut width_min = 0.0f32;

        self.sizes_changed = false;
        let mut children_changed = false;

        for i in state.children() {
            let state = self.states.get_mut(&i.id()).unwrap();
            let (size, s, c) = state.size(i);
            self.sizes_changed |= s;
            children_changed |= c;
            height_min += size.height.min;
            width_min = width_min.max(size.width.min);
        }

        (
            LayoutSize {
                width: LayoutDimension {
                    min: self
                        .size
                        .width
                        .min
                        .map_or(width_min, |min| min.max(width_min)),
                    expand: self.size.width.expand,
                },
                height: LayoutDimension {
                    min: self
                        .size
                        .height
                        .min
                        .map_or(height_min, |min| min.max(height_min)),
                    expand: self.size.height.expand,
                },
            },
            children_changed,
        )
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.layout(state, size);
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        for i in state.children() {
            let state = self.states.get(&i.id()).unwrap();
            let m = Matrix::translate(state.position);
            canvas.save();
            canvas.concat(&m);
            i.draw(canvas);
            canvas.restore();
        }
    }
}
