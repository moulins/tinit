use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};
use std::marker::PhantomData;

use crate::token::{Scope, Token};
use crate::out::Out;

/// An owning handle to a `T` tied to a scope `'s`.
// TODO: impl all useful traits
// TODO: document methods and safety invariants
// Note: with the nightly allocator API, this could be Box<T, Scope<'s>>, probably??.
pub struct Slot<'s, T: ?Sized> {
    ptr: NonNull<T>,
    token: Token<'s>,
    _invariant: PhantomData<*mut T>,
}

impl<'s, T: ?Sized> Deref for Slot<'s, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<'s, T: ?Sized> DerefMut for Slot<'s, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<'s, T: ?Sized> Slot<'s, T> {
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
    pub fn into_parts<'a>(this: Self) -> (Token<'s>, &'a mut T)
    where
        's: 'a,
    {
        let mut this = ManuallyDrop::new(this);
        unsafe {
            (ptr::read(&this.token), this.ptr.as_mut())
        }
    }

    #[inline]
    pub fn into_token(this: Self) -> Token<'s> {
        Self::into_parts(this).0
    }

    #[inline]
    pub fn drop(this: Self) -> Out<'s, T> {
        let (token, val) = Self::into_parts(this);
        let ptr = val as *mut _;
        unsafe {
            ptr::drop_in_place(ptr);
            Out::from_raw(ptr, token)
        }
    }

    #[inline]
    pub fn forget(this: Self) -> Out<'s, T> {
        let (token, val) = Self::into_parts(this);
        let ptr = val as *mut _;
        unsafe { Out::from_raw(ptr, token) }
    }
}

impl<'s, T: ?Sized> Drop for Slot<'s, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.ptr.as_ptr()) }
    }
}

impl<'s, T: ?Sized> From<Slot<'s, T>> for Token<'s> {
    #[inline]
    fn from(slot: Slot<'s, T>) -> Self {
        Slot::into_parts(slot).0
    }
}
