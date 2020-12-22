use super::super::{LayoutDimension, LayoutSize, Widget, WidgetWrapper, WrapperState};
use super::container::{ContainerSize, ContainerWidget};
use crate::game::{Canvas, InputEvent};
use crate::skia;
use skia::{Matrix, Size};

pub struct HContainer<T: Widget> {
    inner: Vec<ContainerWidget<T>>,
    pub size: ContainerSize,
}

impl<T: Widget> HContainer<T> {
    pub fn new(inner: Vec<WidgetWrapper<T>>, size: ContainerSize) -> Self {
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

        let space_left = total_space - min;
        let mut offset = 0.0;
        if space_left < 0.0 {
            let s = total_space / self.inner.len().max(1) as f32;
            for i in &mut self.inner {
                i.size.width = s;
                i.position = (offset, 0.0).into();
                offset += s;
                i.size.height = if i.layout_size.height.expand.is_some() {
                    size.height
                } else {
                    i.layout_size.height.min.min(size.height)
                }
            }
        } else {
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
}

impl<T: Widget> Widget for HContainer<T> {
    fn update(&mut self, wrap: &mut WrapperState) {
        for i in &mut self.inner {
            i.inner.update();
        }
    }

    fn input(&mut self, event: &InputEvent, size: Size) -> bool {
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

    fn size(&mut self) -> LayoutSize {
        let mut size = 0.0f32;
        let mut min = 0.0f32;

        let mut height = 0.0f32;
        let mut height_min = 0.0f32;
        for i in &mut self.inner {
            let s = i.inner.size();
            i.layout_size = s;
            size += s.width.size;
            min += s.width.min;
            height = height.max(s.height.size);
            height_min = height_min.max(s.height.min);
        }

        LayoutSize {
            width: LayoutDimension {
                size,
                min: self.size.width.min.map_or(min, |pmin| pmin.max(min)),
                expand: self.size.width.expand,
            },
            height: LayoutDimension {
                size: height,
                min: self
                    .size
                    .height
                    .min
                    .map_or(height_min, |pmin| pmin.max(height_min)),
                expand: self.size.height.expand,
            },
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, size: Size) {
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
