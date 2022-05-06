use core::mem::MaybeUninit;
use core::ptr::{self, NonNull};
use core::slice;
use std::marker::PhantomData;

use crate::token::{Scope, Token};
use crate::slot::Slot;

/// An owning handle to an uninitialized `T` tied to a scope `'s`.
///
/// Morally equivalent to [`Slot<'s, MaybeUninit<T>>`][Slot], but supports dynamically sized types.
// TODO: document methods and safety invariant.
pub struct Out<'s, T: ?Sized> {
    ptr: NonNull<T>,
    token: Token<'s>,
    _invariant: PhantomData<*mut T>,
}

impl<'s, T: ?Sized> Out<'s, T> {
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: *mut T, token: Token<'s>) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            token,
            _invariant: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn forge(scope: Scope<'s>, ptr: *mut T) -> Self {
        unsafe { Self::from_raw(ptr, Token::new(scope)) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    #[inline]
    pub unsafe fn assume_init(self) -> Slot<'s, T> {
        unsafe { Slot::from_raw(self.ptr.as_ptr(), self.token) }
    }
}

impl<'s, T> Out<'s, T> {
    #[inline]
    pub fn as_uninit(&self) -> &MaybeUninit<T>
    where
        T: Sized,
    {
        unsafe { self.ptr.cast::<MaybeUninit<T>>().as_ref() }
    }

    #[inline]
    pub fn as_uninit_mut(&mut self) -> &mut MaybeUninit<T>
    where
        T: Sized,
    {
        unsafe { self.ptr.cast::<MaybeUninit<T>>().as_mut() }
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
