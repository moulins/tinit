use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::init::Init;
use crate::place::Place;
use crate::Slot;

type Invariant<T> = PhantomData<core::cell::Cell<T>>;
type Covariant<T> = PhantomData<T>;

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
    pub fn borrow<P: Place>(&self, place: &'s mut P) -> LeasedMem<'s, P::Type> {
        LeasedMem {
            ptr: unsafe { NonNull::new_unchecked(place.as_mut_ptr()) },
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
    pub unsafe fn borrow_ptr<T: ?Sized>(&self, ptr: *mut T) -> LeasedMem<'s, T> {
        LeasedMem {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }
}

macro_rules! impl_place_trait {
    ($name:ident) => {
        unsafe impl<'s, T: ?Sized> Place for $name<'s, T> {
            type Type = T;

            type Init = Init<Self>;

            #[inline(always)]
            fn as_ptr(&self) -> *const Self::Type {
                self.ptr.as_ptr() as *const _
            }

            #[inline(always)]
            fn as_mut_ptr(&mut self) -> *mut Self::Type {
                self.ptr.as_ptr()
            }

            #[inline(always)]
            unsafe fn assume_init(self) -> Self::Init {
                unsafe { Init::from_place(self) }
            }
        }
    };
}

impl_place_trait!(OwnedMem);
impl_place_trait!(LeasedMem);
