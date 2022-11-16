use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::place::{IntoPlace, Place};
use crate::mem::Mem;

// TODO: impl all useful traits
// TODO: document methods and safety invariants
#[repr(transparent)]
pub struct Init<P: Place>(P);

impl<P: Place> Deref for Init<P> {
    type Target = P::Type;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        // SAFETY: per our invariants, we contain a valid `T`.
        unsafe { self.0.raw_ref().into_ref() }
    }
}

impl<P: Place> DerefMut for Init<P> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: per our invariants, we contain a valid `T`.
        unsafe { self.0.raw_mut().into_mut() }
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
        unsafe { place.raw_mut().drop_in_place() };
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
        (unsafe { place.raw_ref().read() }, place)
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
        unsafe { Self::forget(this).finalize() }
    }
}

impl<P: Place> Drop for Init<P> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `place` contains a valid T, and is never accessed after this line.
        unsafe { self.0.raw_mut().drop_in_place() }
    }
}
