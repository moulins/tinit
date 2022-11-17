use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::mem::Mem;
use crate::place::{Place, Slot};

// TODO: impl all useful traits
// TODO: document methods and safety invariants
#[repr(transparent)]
pub struct Init<P: Place> {
    place: P,
    // We logically own the value stored in the place.
    _marker: PhantomData<P::Init>,
}

impl<P: Place> Deref for Init<P> {
    type Target = P::Target;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        // SAFETY: per our invariants, we contain a valid `T`.
        unsafe { self.place.deref_place().into_ref() }
    }
}

impl<P: Place> DerefMut for Init<P> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: per our invariants, we contain a valid `T`.
        unsafe { self.place.deref_place_mut().into_mut() }
    }
}

impl<'s, T> Init<Mem<'s, T>> {
    #[inline(always)]
    pub fn new_in(uninit: &'s mut MaybeUninit<T>, value: T) -> Self {
        Mem::new(uninit.into()).set(value)
    }
}

impl<T: ?Sized, P> Init<P>
where
    P: Place<Target = T>,
{
    #[inline(always)]
    pub unsafe fn from_place(place: P) -> Self {
        Self {
            place,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn drop(this: Self) -> P {
        let mut place = Self::forget(this);
        // SAFETY: `place` contains a valid T, and becomes logically uninit.
        unsafe { place.deref_place_mut().drop_in_place() };
        place
    }

    #[inline(always)]
    pub fn forget(this: Self) -> P {
        // Disable the drop impl.
        let this = ManuallyDrop::new(this);
        // SAFETY: `this` isn't accessed nor dropped after this line.
        unsafe { ptr::read(&this.place) }
    }

    #[inline(always)]
    pub fn take(this: Self) -> T
    where
        T: Sized,
    {
        Self::take_full(this).0
    }

    #[inline(always)]
    pub fn take_full(this: Self) -> (T, P)
    where
        T: Sized,
    {
        let place = Self::forget(this);
        // SAFETY: `place` contains a valid T, and becomes logically uninit.
        (unsafe { place.deref_place().read() }, place)
    }

    // TODO: see Place::leak
    #[inline(always)]
    pub fn leak<'a>(this: Self) -> &'a mut T
    where
        P: 'a,
    {
        // SAFETY: `place` contains a valid `T`.
        unsafe { Self::forget(this).leak().into_mut() }
    }

    #[inline(always)]
    pub fn finalize(this: Self) -> P::Init {
        // SAFETY: `place` contains a valid `T`.
        unsafe { Self::forget(this).assume_init() }
    }
}

impl<P: Place> Drop for Init<P> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `place` contains a valid T, and is never accessed after this line.
        unsafe { self.place.deref_place_mut().drop_in_place() }
    }
}
