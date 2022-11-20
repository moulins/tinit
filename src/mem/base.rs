use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::panic::{RefUnwindSafe, UnwindSafe};
use core::ptr::NonNull;

use crate::init::Init;
use crate::place::{Place, Slot};
use crate::uninit::{UninitMut, UninitRef};


// TODO: document
#[repr(transparent)]
pub struct Mem<'scope, T: ?Sized> {
    // Ideally this'd be bivariant in T, but bivariance doesn't exist in Rust.
    ptr: NonNull<T>,
    // We actually stores any T.
    _marker: PhantomData<&'scope ()>,
}

impl<'s, T: ?Sized> Mem<'s, T> {
    #[inline(always)]
    pub fn new(mut uninit: UninitMut<'s, T>) -> Self {
        Self {
            ptr: uninit.as_non_null(),
            _marker: PhantomData,
        }
    }

    // SAFETY: ptr must be live and unaliased during `'s`.
    #[inline(always)]
    pub unsafe fn from_raw(uninit: *mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(uninit) },
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_raw(self) -> *mut T {
        self.ptr.as_ptr()
    }

}

unsafe impl<'s, T: ?Sized> Place for Mem<'s, T> {
    type Target = T;
    type Init = Init<Self>;

    #[inline(always)]
    fn deref_place(&self) -> UninitRef<'_, Self::Target> {
        unsafe { UninitRef::new_unchecked(self.ptr.as_ptr()) }
    }

    #[inline(always)]
    fn deref_place_mut(&mut self) -> UninitMut<'_, Self::Target> {
        unsafe { UninitMut::new_unchecked(self.ptr.as_ptr()) }
    }

    #[inline(always)]
    unsafe fn assume_init(self) -> Self::Init {
        unsafe { Init::new_unchecked(self) }
    }
}

// Unconditionally implement a bunch of auto-traits, as we
// don't care about the actual type inside.
unsafe impl<'s, T: ?Sized> Send for Mem<'s, T> {}
unsafe impl<'s, T: ?Sized> Sync for Mem<'s, T> {}
impl<'s, T: ?Sized> Unpin for Mem<'s, T> {}
impl<'s, T: ?Sized> UnwindSafe for Mem<'s, T> {}
impl<'s, T: ?Sized> RefUnwindSafe for Mem<'s, T> {}

impl<'s, T> Slot<Init<Mem<'s, T>>> for &'s mut MaybeUninit<T> {
    type Place = Mem<'s, T>;

    #[inline(always)]
    fn into_place(self) -> Self::Place {
        Mem::new(UninitMut::from(self))
    }
}

impl<'s, T> Slot<Init<Mem<'s, T>>> for UninitMut<'s, T> {
    type Place = Mem<'s, T>;

    #[inline(always)]
    fn into_place(self) -> Self::Place {
        Mem::new(self)
    }
}

// TODO: impl Slot for &'s mut [MU<T>]
