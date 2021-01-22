use std::any::{TypeId, Any};

use uany::{UnsafeAny, UnsafeAnyExt};
use unsafe_any as uany;


struct ResourceStack<A: ?Sized = dyn UnsafeAny>
where A: UnsafeAnyExt {
    stack: Vec<(TypeId, Box<A>)>,
}

impl ResourceStack {
    pub fn new() -> Self {
        Self {
            stack: vec![],
        }
    }
}

impl<A: UnsafeAnyExt + ?Sized> ResourceStack<A> {
    pub fn push<T: Any + Implements<A>>(&mut self, val: T) {
        self.stack.push((TypeId::of::<T>(), val.into_object()));
    }

    pub fn get<T: Any + Implements<A>>(&self) -> Option<&T> {
        let target = TypeId::of::<T>();
        self.stack.iter().rev().find(|(id, _)| target == *id).map(|e| {
            unsafe {
                e.1.downcast_ref_unchecked::<T>()
            }
        })
    }
}

pub unsafe trait Implements<A: ?Sized + UnsafeAnyExt> {
    fn into_object(self) -> Box<A>;
}

unsafe impl<T: UnsafeAny> Implements<dyn UnsafeAny> for T {
    fn into_object(self) -> Box<dyn UnsafeAny> { Box::new(self) }
}