/*! Utility for resource management where there is one "owner" and multiple
   users. Used for state management, resource management, audio, and other
   things.

   Contains `unsafe` blocks, borrowing safety backed by RefCell. Is memory safe
   and cannot be exploited to get two or more mutable references to the resource
   as far as testing done by the author can tell.
*/

mod stack;

pub use stack::ResourceStack;

use std::cell::{Ref, RefCell, RefMut};
use std::intrinsics::transmute;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use crate::game::ID;

/// The struct owning the resource.
///
/// This by itself serves no meaning to the user of the API, except for
/// ownership, where this struct should be alive for as long as the resource is
/// to be used. Else, the users of the resource would simply get `None`s.
pub struct ResourceHoster<T> {
    resource: Rc<RefCell<ResourceWrapper<T>>>,
}

struct ResourceWrapper<T> {
    resource: T,
    id: ID,
}

impl<T> ResourceHoster<T> {
    /// Creates the resource.
    pub fn new(resource: T) -> Self {
        Self {
            resource: Rc::new(RefCell::new(ResourceWrapper {
                resource,
                id: ID::next(),
            })),
        }
    }

    /// Creates an user of the resource.
    pub fn new_user(&self) -> ResourceUser<T> {
        ResourceUser {
            resource: Rc::downgrade(&self.resource),
        }
    }
}

/// An user of a resource.
///
/// Functionally, it's no more than a `Weak<RefCell<T>>` where `T` is the
/// resource, but the types its methods return is rather useful.
pub struct ResourceUser<T> {
    resource: Weak<RefCell<ResourceWrapper<T>>>,
}

impl<T> PartialEq for ResourceUser<T> {
    fn eq(&self, other: &Self) -> bool {
        let id = self.resource.upgrade().map(|e| e.borrow().id);
        let other_id = other.resource.upgrade().map(|e| e.borrow().id);
        id.is_some() && other_id.is_some() && id == other_id
    }
}

impl<T> Clone for ResourceUser<T> {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
        }
    }
}

impl<T> ResourceUser<T> {
    /// Creates a "null" resource user.
    pub fn new_none() -> Self {
        Self {
            resource: Weak::new(),
        }
    }

    /// Tries accessing the resource.
    ///
    /// Returns a `None` if the resource owner has been dropped.
    /// Functionally is the same as `RefCell::borrow`.
    ///
    /// # Panics
    /// Panics if the resource is currently being mutably borrowed.
    pub fn try_access(&self) -> Option<ResourceUsage<T>> {
        let rc = self.resource.upgrade()?;
        // SAFETY: Transmutation of `Ref` here is okay, as we are clinging onto
        // the corresponding `Rc`, which means the data `Ref` is pointing to is
        // valid for as long as we require it to.
        unsafe {
            let val: Ref<'_, ResourceWrapper<T>> = transmute(rc.borrow());
            let usage = transmute(&val.deref().resource);
            let r = ResourceRefHolder { _val: val, _rc: rc };
            Some(ResourceUsage { usage, holder: r })
        }
    }

    /// Tries mutably accessing the resource.
    ///
    /// Returns a `None` if the resource owner has been dropped.
    /// Functionally is the same as `RefCell::borrow_mut`.
    ///
    /// # Panics
    /// Panics if the resource is currently being borrowed, be it mutable or
    /// not.
    pub fn try_access_mut(&self) -> Option<ResourceUsageMut<T>> {
        let rc = self.resource.upgrade()?;
        // SAFETY: Transmutation of `RefMut` here is okay, as we are clinging
        // onto the corresponding `Rc`, which means the data `Ref` is pointing
        // to is valid for as long as we require it to.
        unsafe {
            let mut val: RefMut<'_, ResourceWrapper<T>> =
                transmute(rc.borrow_mut());
            let usage = transmute(&mut val.deref_mut().resource);
            let r = ResourceRefMutHolder { _val: val, _rc: rc };
            Some(ResourceUsageMut { usage, holder: r })
        }
    }
}

/// Resource usage, returned by `ResourceUser::try_access`.
///
/// The `R` type parameter is the type of the struct the resource user is
/// pointing to, while the `T` type parameter is the type borrowed from `R`.
/// `T` defaults to `R`.
///
/// Functionally is the same as `Ref`, but also holds a strong reference to the
/// resource for obvious lifetime reasons. As such, it releases the runtime
/// lock on the `RefCell` when dropped.
///
/// Implements `Deref` as this is a smart pointer of some sort.
pub struct ResourceUsage<'a, R, T = R> {
    usage: &'a T,
    holder: ResourceRefHolder<'a, R>,
}

struct ResourceRefHolder<'a, T> {
    // SAFETY: Notice that the `_rc` field comes AFTER the `_val` field.
    // This is EXTREMELY important, as we always want the `Ref` to drop first.
    _val: Ref<'a, ResourceWrapper<T>>,
    _rc: Rc<RefCell<ResourceWrapper<T>>>,
}

impl<'a, R, T> ResourceUsage<'a, R, T> {
    /// Maps a `ResourceUsage<R, T>` to a `ResourceUsage<R, U>` by calling `f`
    /// on `&T`. Useful for resource users, which returns `Option`s.
    pub fn map<U, F>(self, f: F) -> ResourceUsage<'a, R, U>
    where
        F: FnOnce(&'a T) -> &'a U,
    {
        ResourceUsage {
            usage: f(self.usage),
            holder: self.holder,
        }
    }
}

impl<'a, R, T> Deref for ResourceUsage<'a, R, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.usage
    }
}

/// Mutable resource usage, returned by `ResourceUser::try_access_mut`.
///
/// The `R` type parameter is the type of the struct the resource user is
/// pointing to, while the `T` type parameter is the type borrowed from `R`.
/// `T` defaults to `R`.
///
/// Functionally is the same as `RefMut`, but also holds a strong reference to
/// the resource for obvious lifetime reasons. As such, it releases the runtime
/// lock on the `RefCell` when dropped.
///
/// Implements `Deref` and `DerefMut` as this is a smart pointer of some sort.
pub struct ResourceUsageMut<'a, R, T = R> {
    usage: &'a mut T,
    holder: ResourceRefMutHolder<'a, R>,
}

struct ResourceRefMutHolder<'a, T> {
    // SAFETY: Notice that the `_rc` field comes AFTER the `_val` field.
    // This is EXTREMELY important, as we always want the `RefMut` to drop
    // first.
    _val: RefMut<'a, ResourceWrapper<T>>,
    _rc: Rc<RefCell<ResourceWrapper<T>>>,
}

impl<'a, R, T> ResourceUsageMut<'a, R, T> {
    /// Maps a `ResourceUsage<R, T>` to a `ResourceUsage<R, U>` by calling `f`
    /// on `&mut T`. Useful for resource users, which returns `Option`s.
    pub fn map<U, F>(self, f: F) -> ResourceUsageMut<'a, R, U>
    where
        F: FnOnce(&'a mut T) -> &'a mut U,
    {
        ResourceUsageMut {
            usage: f(self.usage),
            holder: self.holder,
        }
    }
}

impl<'a, R, T> Deref for ResourceUsageMut<'a, R, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.usage
    }
}

impl<'a, R, T> DerefMut for ResourceUsageMut<'a, R, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.usage
    }
}
