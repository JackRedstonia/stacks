/*! Utility for resource management where there is one "owner" and multiple
   users. Used for state management, resource management, audio, and other
   things.

   Contains only three `unsafe` blocks, and the rest of safety backed by
   RefCell. Is memory safe and cannot be exploited to get two or more mutable
   references to the resource as far as testing done by the author can tell.
*/

mod stack;

pub use stack::ResourceStack;

use std::cell::{Ref, RefCell};
use std::{
    cell::RefMut,
    intrinsics::transmute,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

/// The struct owning the resource.
///
/// This by itself serves no meaning to the user of the API, except for
/// ownership, where this struct should be alive for as long as the resource is
/// to be used. Else, the users of the resource would simply get `None`s.
pub struct ResourceHoster<T> {
    resource: Rc<RefCell<T>>,
}

impl<T> ResourceHoster<T> {
    /// Creates the resource.
    pub fn new(resource: T) -> Self {
        Self {
            resource: Rc::new(RefCell::new(resource)),
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
    resource: Weak<RefCell<T>>,
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
        // SAFETY: the `_rc` field is to never be accessed, so this is fine.
        // it's like one part of a struct borrowing the other part, and the
        // struct merely plays the role of being alive until the reference to
        // it dies.
        let val = unsafe { transmute(rc.borrow()) };
        Some(ResourceUsage { _rc: rc, val })
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
        // SAFETY: the `_rc` field is to never be accessed, so this is fine.
        // it's like one part of a struct borrowing the other part, and the
        // struct merely plays the role of being alive until the reference to
        // it dies.
        let val = unsafe { transmute(rc.borrow_mut()) };
        Some(ResourceUsageMut { _rc: rc, val })
    }
}

/// Resource usage, returned by `ResourceUser::try_access`.
///
/// Functionally is the same as `Ref`, but also holds a strong reference to the
/// resource for obvious lifetime reasons. As such, it releases the runtime
/// lock on the `RefCell` when dropped.
///
/// Implements `Deref` as this is a smart pointer of some sort.
pub struct ResourceUsage<'a, T> {
    // SAFETY: This field is to never be accessed.
    // There is simply no reason to do so, as this field is merely here to be
    // dropped when the usage drops.
    _rc: Rc<RefCell<T>>,
    val: Ref<'a, T>,
}

impl<'a, T> Deref for ResourceUsage<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.val
    }
}

/// Mutable resource usage, returned by `ResourceUser::try_access_mut`.
///
/// Functionally is the same as `RefMut`, but also holds a strong reference to
/// the resource for obvious lifetime reasons. As such, it releases the runtime
/// lock on the `RefCell` when dropped.
///
/// Implements `Deref` and `DerefMut` as this is a smart pointer of some sort.
pub struct ResourceUsageMut<'a, T> {
    // SAFETY: This field is to never be accessed.
    // There is simply no reason to do so, as this field is merely here to be
    // dropped when the usage drops.
    _rc: Rc<RefCell<T>>,
    val: RefMut<'a, T>,
}

impl<'a, T> Deref for ResourceUsageMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val.deref()
    }
}

impl<'a, T> DerefMut for ResourceUsageMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.val.deref_mut()
    }
}
