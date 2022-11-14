use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

use crate::init::Init;
use crate::place::Place;

type Invariant<T> = PhantomData<core::cell::Cell<T>>;
type Covariant<T> = PhantomData<T>;

// TODO: document more
#[repr(transparent)]
pub struct Mem<'scope, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: Covariant<&'scope T>,
}

impl<'s, T: ?Sized> Mem<'s, T> {
    // SAFETY: ptr must be live and unaliased during `'s`.
    #[inline(always)]
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn from_uninit(uninit: &'s mut MaybeUninit<T>) -> Self
    where
        T: Sized,
    {
        unsafe { Self::new_unchecked(uninit.as_mut_ptr()) }
    }
}

// TODO: document
#[repr(transparent)]
pub struct ScopedMem<'scope, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: Invariant<&'scope T>,
}

// TODO: document
pub struct Scope<'scope>(Invariant<&'scope ()>);

impl<'s> Scope<'s> {
    #[inline(always)]
    pub(crate) unsafe fn forge() -> Self {
        Scope(PhantomData)
    }

    #[inline(always)]
    pub fn borrow<P: Place>(&self, place: &'s mut P) -> ScopedMem<'s, P::Type> {
        ScopedMem {
            ptr: unsafe { NonNull::new_unchecked(place.as_mut_ptr()) },
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn borrow_uninit<T>(&self, uninit: &'s mut MaybeUninit<T>) -> ScopedMem<'s, T> {
        ScopedMem {
            ptr: NonNull::from(uninit).cast(),
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub unsafe fn borrow_ptr<T: ?Sized>(&self, ptr: *mut T) -> ScopedMem<'s, T> {
        ScopedMem {
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
            unsafe fn finalize(self) -> Self::Init {
                unsafe { Init::from_place(self) }
            }
        }
    };
}

impl_place_trait!(Mem);
impl_place_trait!(ScopedMem);
