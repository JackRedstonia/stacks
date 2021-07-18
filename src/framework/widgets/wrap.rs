use super::{LayoutSize, Widget};

use std::cell::{Ref, RefCell, RefMut};
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use crate::game::{InputEvent, ID};
use crate::skia::{Canvas, Size};

use super::super::resource::ResourceStack;
use super::super::FrameworkState;

pub struct WidgetBorrow<'a, T: 'a + Widget + ?Sized> {
    widget: &'a T,
    _ref: Ref<'a, WrapInner<T>>,
}

impl<'a, T: 'a + Widget + ?Sized> Deref for WidgetBorrow<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.widget
    }
}

pub struct WidgetBorrowMut<'a, T: 'a + Widget + ?Sized> {
    widget: &'a mut T,
    _ref: RefMut<'a, WrapInner<T>>,
}

impl<'a, T: 'a + Widget + ?Sized> Deref for WidgetBorrowMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.widget
    }
}

impl<'a, T: 'a + Widget + ?Sized> DerefMut for WidgetBorrowMut<'a, T> {
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

    /// Convenience function for the lazy.
    /// Most clients should use the much nicer-looking `From<Wrap<T>>`'s
    /// `into` method instead.
    #[allow(clippy::wrong_self_convention)]
    #[deprecated(
        since = "0.0.1",
        note = "Use `From<Wrap<T>>` with `into()` instead"
    )]
    pub fn to_dyn(self) -> Wrap<dyn Widget + 'a> {
        Wrap { inner: self.inner }
    }
}

impl<'a, T: Widget + 'a> From<Wrap<T>> for Wrap<dyn Widget + 'a> {
    fn from(wrap: Wrap<T>) -> Self {
        Wrap { inner: wrap.inner }
    }
}

impl<'a, T: 'a + Widget + ?Sized> Wrap<T> {
    pub fn id(&self) -> ID {
        self.inner.borrow().state.id()
    }

    pub fn inner(&'a self) -> WidgetBorrow<'a, T> {
        let r = self.inner.borrow();
        // This is obviously fine, as `_ref` is never to be accessed.
        let b = unsafe { transmute(&r.inner) };
        WidgetBorrow { widget: b, _ref: r }
    }

    pub fn inner_mut(&'a mut self) -> WidgetBorrowMut<'a, T> {
        let mut r = self.inner.borrow_mut();
        // This is obviously fine, as `_ref` is never to be accessed.
        let b = unsafe { transmute(&mut r.inner) };
        WidgetBorrowMut { widget: b, _ref: r }
    }

    pub fn downgrade(&self) -> WrapWeak<T> {
        WrapWeak {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub fn grab_focus(&self) {
        FrameworkState::grab_focus(self.id())
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

pub struct WrapWeak<T: Widget + ?Sized> {
    inner: Weak<RefCell<WrapInner<T>>>,
}

impl<T: Widget + ?Sized> WrapWeak<T> {
    pub fn upgrade(&self) -> Option<Wrap<T>> {
        self.inner.upgrade().map(|inner| Wrap { inner })
    }
}

impl<T: Widget + ?Sized> Clone for WrapWeak<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Weak::clone(&self.inner),
        }
    }
}

pub trait Wrappable<'a, T: Widget> {
    fn wrap(self) -> Wrap<T>;
}

impl<'a, T: 'a + Widget> Wrappable<'a, T> for T {
    fn wrap(self) -> Wrap<T> {
        Wrap::new(self)
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
    is_hovered: bool,
    was_hovered: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self {
            id: ID::next(),
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
}

impl Default for WidgetState {
    fn default() -> Self {
        Self::new()
    }
}
