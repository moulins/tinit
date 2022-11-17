use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::init::Init;
use crate::place::Place;
use crate::uninit::{UninitMut, UninitRef};

// Note: ideally, this would be bivariant, but bivariant pointers don't exist in Rust.
type Pointer<T> = NonNull<T>;

// TODO: document
#[repr(transparent)]
pub struct Mem<'scope, T: ?Sized> {
    ptr: Pointer<T>,
    // We don't actually store any T
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
    ptr: Pointer<T>,
    // The backing memory is logically consumed when the scope is exited.
    _marker: PhantomData<fn(Mem<'scope, T>) -> &'scope ()>,
}

macro_rules! impl_place_trait {
    ($name:ident) => {
        unsafe impl<'s, T: ?Sized> Place for $name<'s, T> {
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
                unsafe { Init::from_place(self) }
            }
        }
    };
}

impl_place_trait!(Mem);
impl_place_trait!(ScopedMem);

// TODO: document
pub struct Scope<'scope>(PhantomData<fn(&'scope ()) -> &'scope ()>);

impl<'s> Scope<'s> {
    #[inline(always)]
    pub(crate) unsafe fn forge() -> Self {
        Scope(PhantomData)
    }

    #[inline(always)]
    pub fn borrow<P: Place>(&self, slot: &'s mut P) -> ScopedMem<'s, P::Target> {
        ScopedMem {
            ptr: slot.deref_place_mut().as_non_null(),
            _marker: PhantomData,
        }
    }
}
