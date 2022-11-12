use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::place::{IntoPlace, Place};
use crate::Slot;
use crate::Uninit;

// TODO: impl all useful traits
// TODO: document methods and safety invariants
#[repr(transparent)]
pub struct Init<P: Place>(P);

impl<P: Place> Deref for Init<P> {
    type Target = P::Type;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        // SAFETY: per our invariants, we contain a valid `T`.
        unsafe { &*self.0.as_ptr() }
    }
}

impl<P: Place> DerefMut for Init<P> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: per our invariants, we contain a valid `T`.
        unsafe { &mut *self.0.as_mut_ptr() }
    }
}

impl<'s, T> Init<Uninit<'s, T>> {
    #[inline(always)]
    pub fn new_in(slot: Slot<'s, T>, value: T) -> Self {
        Uninit::from_slot(slot).set(value)
    }
}

impl<T: ?Sized, P> Init<P>
where
    P: Place<Type = T>,
{
    #[inline(always)]
    pub unsafe fn from_place(place: P) -> Self {
        Self(place)
    }

    #[inline]
    pub fn drop(this: Self) -> P {
        let mut place = Self::forget(this);
        // SAFETY: `place` contains a valid T, and becomes logically uninit.
        unsafe { ptr::drop_in_place(place.as_mut_ptr()) };
        place
    }

    #[inline(always)]
    pub fn forget(this: Self) -> P {
        // Disable the drop impl.
        let this = ManuallyDrop::new(this);
        // SAFETY: `this` isn't accessed nor dropped after this line.
        unsafe { ptr::read(&this.0) }
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
        let value = unsafe { ptr::read(place.as_ptr()) };
        (value, place)
    }

    #[inline(always)]
    pub fn leak<'a>(this: Self) -> &'a mut T
    where
        P: 'a,
    {
        let mut place = Self::forget(this);
        // SAFETY: `place` contains a valid `T`, and the lifetime is properly constrained.
        unsafe { &mut *place.as_mut_ptr() }
    }

    #[inline(always)]
    pub fn finalize(this: Self) -> P::Init {
        // SAFETY: `place` contains a valid T
        unsafe { Self::forget(this).assume_init() }
    }
}

impl<P: Place> Drop for Init<P> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `place` contains a valid T, and is never accessed after this line.
        unsafe { ptr::drop_in_place(self.0.as_mut_ptr()) }
    }
}
