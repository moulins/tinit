use core::marker::PhantomData;

use crate::init::Init;
use crate::place::Place;
use super::Mem;

// TODO: document
#[repr(transparent)]
pub struct ScopedMem<'scope, T: ?Sized> {
    mem: Mem<'scope, T>,
    // The backing memory is logically consumed when the scope is exited.
    _marker: PhantomData<fn(Mem<'scope, T>) -> ()>,
}

unsafe impl<'s, T: ?Sized> Place for ScopedMem<'s, T> {
    type Target = T;
    type Init = Init<Self>;

    impl_place_deref!(use mem);

    #[inline(always)]
    unsafe fn assume_init(self) -> Self::Init {
        unsafe { Init::new_unchecked(self) }
    }
}

// TODO: document
pub struct Scope<'scope>(PhantomData<fn(&'scope ()) -> &'scope ()>);

impl<'s> Scope<'s> {
    #[inline(always)]
    pub(crate) unsafe fn forge() -> Self {
        Scope(PhantomData)
    }

    #[inline(always)]
    pub fn borrow<P: Place>(&self, place: &'s mut P) -> ScopedMem<'s, P::Target> {
        ScopedMem {
            mem: Mem::new(place.deref_place_mut()),
            _marker: PhantomData,
        }
    }
}
