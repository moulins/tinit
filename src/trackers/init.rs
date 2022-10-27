use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::Slot;
use crate::mem::{Mem, OwnedMem};

use super::Uninit;

// TODO: impl all useful traits
// TODO: document methods and safety invariants
#[repr(transparent)]
pub struct Init<M: Mem>(M);

impl<M: Mem> Deref for Init<M> {
    type Target = M::Shape;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        // SAFETY: `mem` contains a valid `T`.
        unsafe { &*self.0.as_ptr() }
    }
}

impl<M: Mem> DerefMut for Init<M> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `mem` contains a valid `T`.
        unsafe { &mut *self.0.as_mut_ptr() }
    }
}

impl<'s, T> Init<OwnedMem<'s, T>> {
    #[inline(always)]
    pub fn new_in(slot: Slot<'s, T>, value: T) -> Self {
        Uninit::from(slot).set(value)
    }
}

impl<T: ?Sized, M> Init<M>
where
    M: Mem<Shape = T>,
{
    #[inline(always)]
    pub unsafe fn from_mem(mem: M) -> Self {
        Self(mem)
    }

    #[inline(always)]
    fn into_mem(this: Self) -> M {
        // Disable the drop impl.
        let this = ManuallyDrop::new(this);
        // SAFETY: `this` isn't accessed nor dropped after this line.
        unsafe { ptr::read(&this.0) }
    }

    #[inline]
    pub fn drop(this: Self) -> Uninit<M> {
        let mut mem = Self::into_mem(this);
        // SAFETY: `mem` contains a valid T, and becomes logically uninit.
        unsafe { ptr::drop_in_place(mem.as_mut_ptr()) }
        mem.into()
    }

    #[inline(always)]
    pub fn forget(this: Self) -> Uninit<M> {
        Self::into_mem(this).into()
    }

    #[inline(always)]
    pub fn take(this: Self) -> T
    where
        T: Sized,
    {
        Self::take_full(this).0
    }

    #[inline(always)]
    pub fn take_full(this: Self) -> (T, Uninit<M>)
    where
        T: Sized,
    {
        let mem = Self::into_mem(this);
        // SAFETY: `mem` contains a valid T, and becomes logically uninit.
        let value = unsafe { ptr::read(mem.as_ptr()) };
        (value, mem.into())
    }

    #[inline(always)]
    pub fn leak<'a>(this: Self) -> &'a mut T
    where
        M: 'a,
    {
        let mut mem = Self::into_mem(this);
        // SAFETY: `mem` contains a valid `T`, and the lifetime is properly constrained.
        unsafe { &mut *mem.as_mut_ptr() }
    }
}

impl<M: Mem> Drop for Init<M> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.0.as_mut_ptr()) }
    }
}
