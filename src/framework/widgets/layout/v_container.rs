use super::super::{LayoutDimension, LayoutSize, Widget, Wrap, WrapState};
use super::container::{ContainerSize, ContainerWidget};
use crate::game::{Canvas, InputEvent};
use crate::skia;
use skia::{Matrix, Size};

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

        let space_left = total_space - min;
        let mut offset = 0.0;
        if space_left < 0.0 {
            let s = total_space / self.inner.len().max(1) as f32;
            for i in &mut self.inner {
                i.size.height = s;
                i.position = (0.0, offset).into();
                offset += s;
                i.size.width = if i.layout_size.width.expand.is_some() {
                    size.width
                } else {
                    i.layout_size.width.min.min(size.width)
                }
            }
        } else {
            for i in &mut self.inner {
                i.size.height = i.layout_size.height.min;
                if let Some(e) = i.layout_size.height.expand {
                    i.size.height += space_left * e / expand;
                }
                i.position = (0.0, offset).into();
                offset += i.size.height;
                i.size.width = if i.layout_size.width.expand.is_some() {
                    size.width
                } else {
                    i.layout_size.width.min.min(size.width)
                }
            }
        }
    }
}

impl<T: Widget> Widget for VContainer<T> {
    fn update(&mut self, _wrap: &mut WrapState) {
        for i in &mut self.inner {
            i.inner.update();
        }
    }

    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent, size: Size) -> bool {
        self.layout(size);
        self.inner.iter_mut().rev().any(|i| {
            let m = Matrix::translate(i.position);
            if let Some(event) = event.reverse_map_position(m.invert().unwrap()) {
                i.inner.input(&event, i.size)
            } else {
                false
            }
        })
    }

    fn size(&mut self, _wrap: &mut WrapState) -> LayoutSize {
        let mut size = 0.0f32;
        let mut min = 0.0f32;

        let mut width = 0.0f32;
        let mut width_min = 0.0f32;
        for i in &mut self.inner {
            let s = i.inner.size();
            i.layout_size = s;
            size += s.height.size;
            min += s.height.min;
            width = width.max(s.width.size);
            width_min = width_min.max(s.width.min);
        }

        LayoutSize {
            width: LayoutDimension {
                size,
                min: self.size.width.min.map_or(min, |pmin| pmin.max(width_min)),
                expand: self.size.width.expand,
            },
            height: LayoutDimension {
                size: width,
                min: self.size.height.min.map_or(width_min, |pmin| pmin.max(min)),
                expand: self.size.height.expand,
            },
        }
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas, size: Size) {
        self.layout(size);
        for i in &mut self.inner {
            let m = Matrix::translate(i.position);
            canvas.save();
            canvas.concat(m);
            i.inner.draw(canvas, i.size);
            canvas.restore();
        }
    }
}
