use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

use crate::scope::{Scope, ScopedPtr};
use crate::out::Out;

/// An owning handle to a `T` tied to a scope `'s`.
// TODO: impl all useful traits
// TODO: document methods and safety invariants
// Note: with the nightly allocator API, this could be Box<T, Scope<'s>>, probably??.
pub struct Slot<'s, T: ?Sized>(ScopedPtr<'s, T>);

impl<'s, T: ?Sized> Deref for Slot<'s, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<'s, T: ?Sized> DerefMut for Slot<'s, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

impl<'s, T: ?Sized> Slot<'s, T> {
    #[inline]
    pub(crate) unsafe fn from_raw(raw: ScopedPtr<'s, T>) -> Self {
        Self(raw)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptr: *mut T, scope: Scope<'s>) -> Self {
        Self(scope.with_ptr(NonNull::new_unchecked(ptr)))
    }

    #[inline]
    pub fn drop(this: Self) -> Out<'s, T> {
        unsafe { Out::from_raw(this.0) }
        // implicit drop of this, and thus of contained T
    }

    #[inline]
    pub fn leak<'a>(this: Self) -> &'a mut T
    where
        's: 'a,
    {
        let mut this = ManuallyDrop::new(this);
        unsafe { this.0.as_mut() }
    }

    #[inline]
    pub fn forget(this: Self) -> Out<'s, T> {
        let this = ManuallyDrop::new(this);
        unsafe { Out::from_raw(this.0) }
    }
}

impl<'s, T: ?Sized> Drop for Slot<'s, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.0.as_ptr()) }
    }
}
