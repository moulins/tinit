use core::mem::MaybeUninit;
use core::ptr::{self, NonNull};
use core::slice;

use crate::scope::{Scope, ScopedPtr};
use crate::slot::Slot;

/// An owning handle to an uninitialized `T` tied to a scope `'s`.
///
/// Morally equivalent to [`Slot<'s, MaybeUninit<T>>`][Slot], but supports dynamically sized types.
// TODO: document methods and safety invariant.
// Note that this could be contravariant in T, but this isn't possible.
pub struct Out<'s, T: ?Sized>(ScopedPtr<'s, T>);

impl<'s, T: ?Sized> Out<'s, T> {
    #[inline]
    pub(crate) unsafe fn from_raw(raw: ScopedPtr<'s, T>) -> Self {
        Self(raw)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptr: *mut T, scope: Scope<'s>) -> Self {
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Self(scope.with_ptr(ptr))
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_ptr()
    }

    #[inline]
    pub unsafe fn assume_init(self) -> Slot<'s, T> {
        unsafe { Slot::from_raw(self.0) }
    }
}

impl<'s, T> Out<'s, T> {
    #[inline]
    pub fn as_uninit(&self) -> &MaybeUninit<T>
    where
        T: Sized,
    {
        unsafe { self.0.cast::<MaybeUninit<T>>().as_ref() }
    }

    #[inline]
    pub fn as_uninit_mut(&mut self) -> &mut MaybeUninit<T>
    where
        T: Sized,
    {
        unsafe { self.0.cast::<MaybeUninit<T>>().as_mut() }
    }

    #[inline]
    pub fn fill(mut self, value: T) -> Slot<'s, T>
    where
        T: Sized,
    {
        unsafe {
            ptr::write(self.as_mut_ptr(), value);
            self.assume_init()
        }
    }
}

impl<'s, T> Out<'s, [T]> {
    #[inline]
    pub fn as_uninit_slice(&mut self) -> &mut [MaybeUninit<T>] {
        let len = <[T] as crate::SliceLike>::len(self);
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr() as *mut _, len) }
    }

    #[inline]
    pub fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<T>] {
        let len = <[T] as crate::SliceLike>::len(self);
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr() as *mut _, len) }
    }
}
