use crate::prelude::*;
use skia::{Canvas, Matrix, Size};

use super::container::{ContainerSize, ContainerWidget};

pub struct HContainer<T: Widget> {
    inner: Vec<ContainerWidget<T>>,
    sizes_changed: bool,
    pub size: ContainerSize,
}

impl<T: Widget> HContainer<T> {
    pub fn new<I, W>(inner: I, size: ContainerSize) -> Self
    where
        W: Into<Wrap<T>>,
        I: IntoIterator<Item = W>,
    {
        FrameworkState::request_load();
        Self {
            inner: inner
                .into_iter()
                .map(Into::into)
                .map(ContainerWidget::new)
                .collect(),
            sizes_changed: false,
            size,
        }
    }

    fn layout(&mut self, size: Size) {
        let total_space = size.width;

        let mut min = 0.0f32;
        let mut expand = 0.0f32;

        for i in &mut self.inner {
            min += i.layout_size.width.min;
            if let Some(e) = i.layout_size.width.expand {
                expand += e;
            }
        }

        let space_left = (total_space - min).max(0.0);
        let mut offset = 0.0;
        for i in &mut self.inner {
            let mut width = i.layout_size.width.min;
            if let Some(e) = i.layout_size.width.expand {
                width += space_left * e / expand;
            }
            i.position.set(offset, 0.0);
            offset += width;
            let height = if i.layout_size.height.expand.is_some() {
                size.height
            } else {
                i.layout_size.height.min.min(size.height)
            };
            i.maybe_set_size(Size::new(width, height));
        }
    }
}

impl<T: Widget> Widget for HContainer<T> {
    fn load(&mut self, _wrap: &mut WidgetState, stack: &mut ResourceStack) {
        for i in &mut self.inner {
            i.inner.load(stack);
        }
    }

    fn update(&mut self, _wrap: &mut WidgetState) {
        for i in &mut self.inner {
            i.inner.update();
        }
    }

    fn input(&mut self, _wrap: &mut WidgetState, event: &InputEvent) -> bool {
        let c = event.consumable();
        let mut any = false;
        for i in self.inner.iter_mut().rev() {
            if let Some(event) = event.reverse_map_position(Matrix::translate(i.position)) {
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

    fn size(&mut self, _wrap: &mut WidgetState) -> (LayoutSize, bool) {
        let mut width_min = 0.0f32;
        let mut height_min = 0.0f32;

        self.sizes_changed = false;
        let mut children_changed = false;

        for i in &mut self.inner {
            let (size, s, c) = i.size();
            self.sizes_changed |= s;
            children_changed |= c;
            width_min += size.width.min;
            height_min = height_min.max(size.height.min);
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

    fn set_size(&mut self, _wrap: &mut WidgetState, size: Size) {
        self.layout(size);
    }

    fn draw(&mut self, _wrap: &mut WidgetState, canvas: &mut Canvas) {
        for i in &mut self.inner {
            let m = Matrix::translate(i.position);
            canvas.save();
            canvas.concat(&m);
            i.inner.draw(canvas);
            canvas.restore();
        }
    }

    // fn get(&mut self, _wrap: &mut WrapState, id: ID) -> Option<(&mut dyn Widget, &mut WrapState)> {
    //     for i in &mut self.inner {
    //         let x = i.inner.get(id);
    //         if x.is_some() {
    //             return x;
    //         }
    //     }
    //     None
    // }
}
