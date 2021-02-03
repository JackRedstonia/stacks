use super::{LayoutSize, Widget};

use std::cell::{RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::slice::IterMut;

use crate::game::{InputEvent, ID};
use crate::skia::{Canvas, Size};

use super::super::resource::ResourceStack;
use super::super::FrameworkState;

pub struct WidgetBorrow<'a, T: 'a + Widget + ?Sized> {
    widget: &'a mut T,
    _ref: RefMut<'a, WrapInner<T>>,
}

impl<'a, T: 'a + Widget + ?Sized> Deref for WidgetBorrow<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.widget
    }
}

impl<'a, T: 'a + Widget + ?Sized> DerefMut for WidgetBorrow<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.widget
    }
}

pub struct WidgetChildrenBorrow<'a, T: 'a + Widget + ?Sized> {
    iter: IterMut<'a, Wrap<dyn Widget>>,
    _ref: RefMut<'a, WrapInner<T>>,
}

impl<'a, T: 'a + Widget + ?Sized> Deref for WidgetChildrenBorrow<'a, T> {
    type Target = IterMut<'a, Wrap<dyn Widget>>;

    fn deref(&self) -> &Self::Target {
        &self.iter
    }
}

impl<'a, T: 'a + Widget + ?Sized> DerefMut for WidgetChildrenBorrow<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iter
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

    pub fn children(&'a mut self) -> WidgetChildrenBorrow<'a, T> {
        let mut r = self.inner.borrow_mut();
        let b = unsafe { std::mem::transmute(r.state.children()) };
        WidgetChildrenBorrow { iter: b, _ref: r }
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

impl<T: Widget + ?Sized> Clone for Wrap<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
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

    pub fn children(&mut self) -> IterMut<Wrap<dyn Widget>> {
        self.children.iter_mut()
    }

    pub fn child(&mut self) -> Option<&mut Wrap<dyn Widget>> {
        self.children.first_mut()
    }

    pub fn load<T: Widget + ?Sized>(
        &mut self,
        widget: &mut T,
        stack: &mut ResourceStack,
    ) {
        widget.load(self, stack);
    }

    pub fn update<T: Widget + ?Sized>(&mut self, widget: &mut T) {
        widget.update(self);
    }

    pub fn input<T: Widget + ?Sized>(
        &mut self,
        widget: &mut T,
        event: &InputEvent,
    ) -> bool {
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

    pub fn size<T: Widget + ?Sized>(
        &mut self,
        widget: &mut T,
    ) -> (LayoutSize, bool) {
        widget.size(self)
    }

    pub fn set_size<T: Widget + ?Sized>(&mut self, widget: &mut T, size: Size) {
        widget.set_size(self, size);
    }

    pub fn draw<T: Widget + ?Sized>(
        &mut self,
        widget: &mut T,
        canvas: &mut Canvas,
    ) {
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
        FrameworkState::request_load();
    }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self::new()
    }
}
