use super::super::{LayoutDimension, LayoutSize, Widget, Wrap, WrapState, ID};
use super::container::{ContainerSize, ContainerWidget};
use crate::game::InputEvent;
use crate::skia;
use skia::{Canvas, Matrix, Size};

pub struct HContainer<T: Widget> {
    inner: Vec<ContainerWidget<T>>,
    pub size: ContainerSize,
}

impl<T: Widget> HContainer<T> {
    pub fn new(inner: Vec<Wrap<T>>, size: ContainerSize) -> Self {
        Self {
            inner: inner.into_iter().map(|i| ContainerWidget::new(i)).collect(),
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
            i.size.width = i.layout_size.width.min;
            if let Some(e) = i.layout_size.width.expand {
                i.size.width += space_left * e / expand;
            }
            i.position = (offset, 0.0).into();
            offset += i.size.width;
            i.size.height = if i.layout_size.height.expand.is_some() {
                size.height
            } else {
                i.layout_size.height.min.min(size.height)
            }
        }
    }
}

impl<T: Widget> Widget for HContainer<T> {
    fn update(&mut self, _wrap: &mut WrapState) {
        for i in &mut self.inner {
            i.inner.update();
        }
    }

    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent, size: Size) -> bool {
        self.layout(size);
        self.inner.iter_mut().rev().any(|i| {
            if let Some(event) = event.reverse_map_position(Matrix::translate(i.position)) {
                i.inner.input(&event, i.size)
            } else {
                false
            }
        })
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        let mut width = 0.0f32;
        let mut width_min = 0.0f32;

        let mut height = 0.0f32;
        let mut height_min = 0.0f32;
        for i in &mut self.inner {
            let s = i.inner.size();
            i.layout_size = s;
            width += s.width.size;
            width_min += s.width.min;
            height = height.max(s.height.size);
            height_min = height_min.max(s.height.min);
        }

        LayoutSize {
            width: LayoutDimension {
                size: width,
                min: self
                    .size
                    .width
                    .min
                    .map_or(width_min, |min| min.max(width_min)),
                expand: self.size.width.expand,
            },
            height: LayoutDimension {
                size: height,
                min: self
                    .size
                    .height
                    .min
                    .map_or(height_min, |min| min.max(height_min)),
                expand: self.size.height.expand,
            },
        }
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas, size: Size) {
        self.layout(size);
        for i in &mut self.inner {
            let m = Matrix::translate(i.position);
            canvas.save();
            canvas.concat(&m);
            i.inner.draw(canvas, i.size);
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
