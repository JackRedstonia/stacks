use super::super::{LayoutDimension, LayoutSize, Widget, Wrap, WrapState, ID};
use super::container::{ContainerSize, ContainerWidget};
use crate::game::InputEvent;
use crate::skia;
use skia::{Canvas, Matrix, Size};

pub struct VContainer<T: Widget> {
    inner: Vec<ContainerWidget<T>>,
    pub size: ContainerSize,
}

impl<T: Widget> VContainer<T> {
    pub fn new(inner: Vec<Wrap<T>>, size: ContainerSize) -> Self {
        Self {
            inner: inner.into_iter().map(|i| ContainerWidget::new(i)).collect(),
            size,
        }
    }

    fn layout(&mut self, size: Size) {
        let total_space = size.height;

        let mut min = 0.0f32;
        let mut expand = 0.0f32;

        for i in &mut self.inner {
            min += i.layout_size.height.min;
            if let Some(e) = i.layout_size.height.expand {
                expand += e;
            }
        }

        let space_left = (total_space - min).max(0.0);
        let mut offset = 0.0;
        for i in &mut self.inner {
            let mut height = i.layout_size.height.min;
            if let Some(e) = i.layout_size.height.expand {
                height += space_left * e / expand;
            }
            i.position.set(0.0, offset);
            offset += height;
            let width = if i.layout_size.width.expand.is_some() {
                size.width
            } else {
                i.layout_size.width.min.min(size.width)
            };
            i.inner.set_size(Size::new(width, height));
        }
    }
}

impl<T: Widget> Widget for VContainer<T> {
    fn update(&mut self, _wrap: &mut WrapState) {
        for i in &mut self.inner {
            i.inner.update();
        }
    }

    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent) -> bool {
        self.inner.iter_mut().rev().any(|i| {
            if let Some(event) = event.reverse_map_position(Matrix::translate(i.position)) {
                i.inner.input(&event)
            } else {
                false
            }
        })
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        let mut height = 0.0f32;
        let mut height_min = 0.0f32;
        let mut width = 0.0f32;
        let mut width_min = 0.0f32;

        for i in &mut self.inner {
            let s = i.inner.size();
            i.layout_size = s;
            height += s.height.size;
            height_min += s.height.min;
            width = width.max(s.width.size);
            width_min = width_min.max(s.width.min);
        }

        LayoutSize {
            width: LayoutDimension {
                size: height,
                min: self
                    .size
                    .height
                    .min
                    .map_or(width_min, |min| min.max(height_min)),
                expand: self.size.width.expand,
            },
            height: LayoutDimension {
                size: width,
                min: self
                    .size
                    .width
                    .min
                    .map_or(height_min, |min| min.max(width_min)),
                expand: self.size.height.expand,
            },
        }
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.layout(size);
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        for i in &mut self.inner {
            let m = Matrix::translate(i.position);
            canvas.save();
            canvas.concat(&m);
            i.inner.draw(canvas);
            canvas.restore();
        }
    }

    fn get(&mut self, _wrap: &mut WrapState, id: ID) -> Option<(&mut dyn Widget, &mut WrapState)> {
        for i in &mut self.inner {
            let x = i.inner.get(id);
            if x.is_some() {
                return x;
            }
        }
        None
    }
}
