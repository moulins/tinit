use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::Slot;
use crate::private::Sealed;

type Invariant<T> = PhantomData<T>;
type Covariant<T> = PhantomData<core::cell::Cell<T>>;

/// An untyped chunk of memory, either [`owned`](OwnedMem) or [`leased`](LeasedMem).
pub trait Mem: Sealed {
    type Shape: ?Sized;
    fn as_ptr(&self) -> *const Self::Shape;
    fn as_mut_ptr(&mut self) -> *mut Self::Shape;
}

/// An owned, untyped chunk of memory.
// TODO: document more
#[repr(transparent)]
pub struct OwnedMem<'scope, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: Covariant<&'scope T>,
}

impl<'s, T: ?Sized> OwnedMem<'s, T> {
    // SAFETY: ptr must be live and unaliased during `'s`.
    #[inline(always)]
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn from_slot(r: Slot<'s, T>) -> Self
    where
        T: Sized,
    {
        unsafe { Self::new_unchecked(r.as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn from_leased(mem: LeasedMem<'s, T>) -> Self {
        Self {
            ptr: mem.ptr,
            _marker: PhantomData,
        }
    }
}

/// A leased, untyped chunk of memory.
#[repr(transparent)]
pub struct LeasedMem<'scope, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: Invariant<&'scope T>,
}

// TODO: document
pub struct Lease<'scope>(Invariant<&'scope ()>);

impl<'s> Lease<'s> {
    #[inline(always)]
    pub(crate) unsafe fn forge() -> Self {
        Lease(PhantomData)
    }

    #[inline(always)]
    pub fn borrow<M: Mem>(&self, mem: &'s mut M) -> LeasedMem<'s, M::Shape> {
        LeasedMem {
            ptr: unsafe { NonNull::new_unchecked(mem.as_mut_ptr()) },
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn borrow_slot<T>(&self, slot: Slot<'s, T>) -> LeasedMem<'s, T> {
        LeasedMem {
            ptr: NonNull::from(slot).cast(),
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub unsafe fn borrow_ptr<T>(&self, ptr: *mut T) -> LeasedMem<'s, T> {
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        LeasedMem { ptr, _marker: PhantomData }
    }

    // TODO: do we need a `OwnedMem<'s, T> -> LeasedMem<'s, T>` conversion?

    #[inline(always)]
    pub fn shorten<'a: 's, T>(&self, mem: LeasedMem<'a, T>) -> LeasedMem<'s, T> {
        LeasedMem {
            ptr: mem.ptr,
            _marker: PhantomData,
        }
    }
}

/// A slice-like chunk of untyped memory. Implemented for slices and fixed-sized arrays.
// TODO: document more
pub trait MemSlice: Mem {
    type Elem: Sized;

    fn len(&self) -> usize;
}

macro_rules! impl_mem_traits {
    ($type:ident) => {
        impl<'s, T: ?Sized> Sealed for $type<'s, T> {}

        impl<'s, T: ?Sized> Mem for $type<'s, T> {
            type Shape = T;

            #[inline(always)]
            fn as_ptr(&self) -> *const T {
                self.ptr.as_ptr()
            }

            #[inline(always)]
            fn as_mut_ptr(&mut self) -> *mut T {
                self.ptr.as_ptr()
            }
        }

        impl<'s, const N: usize, T> MemSlice for $type<'s, [T; N]> {
            type Elem = T;

            #[inline(always)]
            fn len(&self) -> usize {
                N
            }
        }

        impl<'s, T> MemSlice for $type<'s, [T]> {
            type Elem = T;

            #[inline(always)]
            fn len(&self) -> usize {
                // SAFETY: self points to some allocated memory.
                unsafe { crate::polyfill::raw_slice_len(self.as_ptr()) }
            }
        }
    };
}

impl_mem_traits!(OwnedMem);
impl_mem_traits!(LeasedMem);
