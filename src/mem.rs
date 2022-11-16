use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::init::Init;
use crate::place::Place;
use crate::uninit::{UninitMut, UninitRef};

type Invariant<'a> = PhantomData<fn(&'a ()) -> &'a ()>;

// TODO: document
#[repr(transparent)]
pub struct Mem<'scope, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'scope T>,
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
    pub unsafe fn new_unchecked(uninit: *mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(uninit) },
            _marker: PhantomData,
        }
    }
}

// TODO: document
#[repr(transparent)]
pub struct ScopedMem<'scope, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<(&'scope T, Invariant<'scope>)>,
}

macro_rules! impl_place_trait {
    ($name:ident) => {
        unsafe impl<'s, T: ?Sized> Place for $name<'s, T> {
            type Type = T;
            type Init = Init<Self>;

            #[inline(always)]
            fn raw_ref(&self) -> UninitRef<'_, Self::Type> {
                unsafe { UninitRef::new_unchecked(self.ptr.as_ptr()) }
            }

            #[inline(always)]
            fn raw_mut(&mut self) -> UninitMut<'_, Self::Type> {
                unsafe { UninitMut::new_unchecked(self.ptr.as_ptr()) }
            }

            #[inline(always)]
            unsafe fn finalize(self) -> Self::Init {
                unsafe { Init::from_place(self) }
            }
        }
    };
}

impl_place_trait!(Mem);
impl_place_trait!(ScopedMem);

// TODO: document
pub struct Scope<'scope>(Invariant<'scope>);

impl<'s> Scope<'s> {
    #[inline(always)]
    pub(crate) unsafe fn forge() -> Self {
        Scope(PhantomData)
    }

    #[inline(always)]
    pub fn borrow<P: Place>(&self, slot: &'s mut P) -> ScopedMem<'s, P::Type> {
        ScopedMem {
            ptr: slot.raw_mut().as_non_null(),
            _marker: PhantomData,
        }
    }
}
