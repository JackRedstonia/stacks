pub mod audio;
pub mod layout;
pub mod shapes;
mod text;
mod transform;

pub use text::{Font, FontStyle, Text};
pub use transform::Transform;

use std::cell::{RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::game::{InputEvent, ID};
use crate::skia::{scalar, Canvas, Matrix, Rect, Size, Vector};

use super::{resource::ResourceStack, FrameworkState};

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

impl Widget for Box<dyn Widget> {
    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack) {
        self.as_mut().load(state, stack);
    }

    fn update(&mut self, state: &mut WidgetState) {
        self.as_mut().update(state);
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        self.as_mut().input(state, event)
    }

    fn hover(&mut self, state: &mut WidgetState) {
        self.as_mut().hover(state);
    }

    fn hover_lost(&mut self, state: &mut WidgetState) {
        self.as_mut().hover_lost(state);
    }

    fn on_child_add(&mut self, child: &mut Wrap<dyn Widget>) {
        self.as_mut().on_child_add(child);
    }

    fn on_child_remove(&mut self, child: &mut Wrap<dyn Widget>) {
        self.as_mut().on_child_remove(child);
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        self.as_mut().size(state)
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.as_mut().set_size(state, size);
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        self.as_mut().draw(state, canvas);
    }
}

pub struct WidgetBorrow<'a, T: 'a + Widget + ?Sized> {
    widget: &'a mut T,
    _ref: RefMut<'a, WrapInner<T>>,
}

impl<'a, T: Widget> Deref for WidgetBorrow<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.widget
    }
}

impl<'a, T: Widget> DerefMut for WidgetBorrow<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.widget
    }
}

pub struct Wrap<T: Widget + ?Sized> {
    inner: Rc<RefCell<WrapInner<T>>>,
}

impl<'a, T: Widget + 'a> Wrap<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(WrapInner::new(inner))),
        }
    }

    fn to_dyn(self) -> Wrap<dyn Widget + 'a> {
        Wrap { inner: self.inner }
    }
}

impl<'a, T: 'a + Widget + ?Sized> Wrap<T> {
    pub fn id(&self) -> ID {
        self.inner.borrow().state.id()
    }

    pub fn inner(&'a mut self) -> WidgetBorrow<'a, T> {
        let mut r = self.inner.borrow_mut();
        // This is obviously fine, as `_ref` is never to be accessed.
        let b = unsafe { std::mem::transmute(&mut r.inner) };
        WidgetBorrow { widget: b, _ref: r }
    }

    pub fn load(&mut self, stack: &mut ResourceStack) {
        let s = &mut *self.inner.borrow_mut();
        let state = &mut s.state;
        let inner = &mut s.inner;
        state.load(inner, stack);
    }

    pub fn update(&mut self) {
        let s = &mut *self.inner.borrow_mut();
        let state = &mut s.state;
        let inner = &mut s.inner;
        state.update(inner);
    }

    pub fn input(&mut self, event: &InputEvent) -> bool {
        let s = &mut *self.inner.borrow_mut();
        let state = &mut s.state;
        let inner = &mut s.inner;
        state.input(inner, event)
    }

    pub fn size(&mut self) -> (LayoutSize, bool) {
        let s = &mut *self.inner.borrow_mut();
        let state = &mut s.state;
        let inner = &mut s.inner;
        state.size(inner)
    }

    pub fn set_size(&mut self, size: Size) {
        let s = &mut *self.inner.borrow_mut();
        let state = &mut s.state;
        let inner = &mut s.inner;
        state.set_size(inner, size);
    }

    pub fn draw(&mut self, canvas: &mut Canvas) {
        let s = &mut *self.inner.borrow_mut();
        let state = &mut s.state;
        let inner = &mut s.inner;
        state.draw(inner, canvas);
    }

    pub fn add_child<E: Widget + 'static>(&mut self, child: Wrap<E>) {
        let s = &mut *self.inner.borrow_mut();
        let mut child = child.to_dyn();
        s.inner.on_child_add(&mut child);
        s.state.add_child_dyn(child);
    }

    pub fn add_child_dyn(&mut self, mut child: Wrap<dyn Widget>) {
        let s = &mut *self.inner.borrow_mut();
        s.inner.on_child_add(&mut child);
        s.state.add_child_dyn(child);
    }

    pub fn with_child<E: Widget + 'static>(mut self, child: Wrap<E>) -> Self {
        self.add_child(child);
        self
    }

    pub fn with_child_dyn(mut self, child: Wrap<dyn Widget>) -> Self {
        self.add_child_dyn(child);
        self
    }
}

impl<T: Widget> From<T> for Wrap<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

pub trait Wrappable<'a, T: Widget> {
    fn wrap(self) -> Wrap<T>;
    fn boxed(self) -> Box<dyn Widget + 'a>;
}

impl<'a, T: 'a + Widget> Wrappable<'a, T> for T {
    fn wrap(self) -> Wrap<T> {
        Wrap::new(self)
    }

    fn boxed(self) -> Box<dyn Widget + 'a> {
        Box::new(self)
    }
}

struct WrapInner<T: Widget + ?Sized> {
    state: WidgetState,
    inner: T,
}

impl<T: Widget> WrapInner<T> {
    fn new(inner: T) -> Self {
        Self {
            inner,
            state: WidgetState::new(),
        }
    }
}

pub struct WidgetState {
    id: ID,
    children: Vec<Wrap<dyn Widget>>,
    is_hovered: bool,
    was_hovered: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self {
            id: ID::next(),
            children: vec![],
            is_hovered: false,
            was_hovered: false,
        }
    }

    pub fn id(&self) -> ID {
        self.id
    }

    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    pub fn children(&mut self) -> core::slice::IterMut<Wrap<dyn Widget>> {
        self.children.iter_mut()
    }

    pub fn child(&mut self) -> Option<&mut Wrap<dyn Widget>> {
        self.children.first_mut()
    }

    pub fn load<T: Widget + ?Sized>(&mut self, widget: &mut T, stack: &mut ResourceStack) {
        widget.load(self, stack);
    }

    pub fn update<T: Widget + ?Sized>(&mut self, widget: &mut T) {
        widget.update(self);
    }

    pub fn input<T: Widget + ?Sized>(&mut self, widget: &mut T, event: &InputEvent) -> bool {
        match event {
            InputEvent::RemoveHoverExcept(id) => {
                let b = self.id == *id;
                if !b {
                    widget.input(self, event);
                    self.is_hovered = false;
                }
                b
            }
            InputEvent::Focused(id, inner) => {
                if self.id == *id {
                    widget.input(self, inner)
                } else {
                    widget.input(self, event)
                }
            }
            event => {
                let b = widget.input(self, event);
                if matches!(event, InputEvent::MouseMove(_)) {
                    self.is_hovered = b;
                }
                b
            }
        }
    }

    pub fn size<T: Widget + ?Sized>(&mut self, widget: &mut T) -> (LayoutSize, bool) {
        widget.size(self)
    }

    pub fn set_size<T: Widget + ?Sized>(&mut self, widget: &mut T, size: Size) {
        widget.set_size(self, size);
    }

    pub fn draw<T: Widget + ?Sized>(&mut self, widget: &mut T, canvas: &mut Canvas) {
        if self.is_hovered != self.was_hovered {
            self.was_hovered = self.is_hovered;
            if self.is_hovered {
                widget.hover(self);
            } else {
                widget.hover_lost(self);
            }
        }
        widget.draw(self, canvas);
    }

    pub fn grab_focus(&self) {
        FrameworkState::grab_focus(self.id);
    }

    pub fn release_focus(&self) {
        FrameworkState::release_focus(self.id);
    }

    pub fn is_focused(&self) -> bool {
        FrameworkState::current_focus()
            .map(|id| self.id == id)
            .unwrap_or(false)
    }

    pub fn add_child<E: 'static + Widget>(&mut self, child: Wrap<E>) {
        self.add_child_dyn(child.to_dyn());
    }

    pub fn add_child_dyn(&mut self, child: Wrap<dyn Widget>) {
        self.children.push(child);
    }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct LayoutSize {
    pub width: LayoutDimension,
    pub height: LayoutDimension,
}

impl LayoutSize {
    pub const ZERO: Self = Self {
        width: LayoutDimension::ZERO,
        height: LayoutDimension::ZERO,
    };

    pub fn min(width: scalar, height: scalar) -> Self {
        Self {
            width: LayoutDimension::min(width),
            height: LayoutDimension::min(height),
        }
    }

    pub fn expand_width_by(mut self, expand: scalar) -> Self {
        self.width.expand = Some(expand);
        self
    }

    pub fn expand_width(mut self) -> Self {
        self.width.expand = Some(1.0);
        self
    }

    pub fn no_expand_width(mut self) -> Self {
        self.width.expand = None;
        self
    }

    pub fn expand_height_by(mut self, expand: scalar) -> Self {
        self.height.expand = Some(expand);
        self
    }

    pub fn expand_height(mut self) -> Self {
        self.height.expand = Some(1.0);
        self
    }

    pub fn no_expand_height(mut self) -> Self {
        self.height.expand = None;
        self
    }

    pub fn get_min(&self) -> Size {
        Size::new(self.width.min, self.height.min)
    }

    pub fn layout_one(&self, size: Size) -> Size {
        Size::new(
            self.width.layout_one(size.width),
            self.height.layout_one(size.height),
        )
    }

    pub fn map(&self, matrix: Matrix) -> Self {
        let min = Self::map_size(self.width.min, self.height.min, matrix);
        let expand_width = self
            .width
            .expand
            .map(|x| matrix.map_vector(Vector::new(x, 0.0)).x.abs());
        let expand_height = self
            .height
            .expand
            .map(|y| matrix.map_vector(Vector::new(0.0, y)).y.abs());
        Self {
            width: LayoutDimension {
                min: min.width.abs(),
                expand: expand_width,
            },
            height: LayoutDimension {
                min: min.height.abs(),
                expand: expand_height,
            },
        }
    }

    fn map_size(width: scalar, height: scalar, matrix: Matrix) -> Size {
        matrix.map_rect(Rect::from_wh(width, height)).0.size()
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct LayoutDimension {
    /// The minimum number of logical pixels this dimension should receive.
    /// May not always be respected by containers, for example when they run out
    /// of space.
    pub min: scalar,

    /// The expansion factor in this dimension.
    /// A Some(x) expresses this widget in this dimension should take up as
    /// much space as it is allowed, with an expansion factor of x.
    /// A None expresses this widget should take up the minimum space,
    /// expressed in the [min](Self::min) field.
    pub expand: Option<scalar>,
}

impl LayoutDimension {
    pub const ZERO: Self = Self {
        min: 0.0,
        expand: None,
    };

    pub fn min(min: scalar) -> Self {
        Self { min, expand: None }
    }

    pub fn with_expand(mut self, expand: scalar) -> Self {
        self.expand = Some(expand);
        self
    }

    pub fn with_expand_one(mut self) -> Self {
        self.expand = Some(1.0);
        self
    }

    pub fn with_no_expand(mut self) -> Self {
        self.expand = None;
        self
    }

    pub fn layout_one(&self, space: scalar) -> scalar {
        if self.expand.is_some() {
            space.max(self.min)
        } else {
            self.min
        }
    }
}
