use core::mem::MaybeUninit;

use crate::{mem::{Mem, OwnedMem}, Slot};

use super::Init;

pub struct Uninit<M: Mem>(M);

impl<T: ?Sized, M> Uninit<M>
where
    M: Mem<Shape = T>,
{
    #[inline(always)]
    pub fn into_mem(self) -> M {
        self.0
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }

    #[inline(always)]
    pub unsafe fn assume_init(self) -> Init<M> {
        unsafe { Init::from_mem(self.0) }
    }
}

impl<T, M> Uninit<M>
where
    M: Mem<Shape = T>,
{
    #[inline(always)]
    pub fn as_uninit(&self) -> &MaybeUninit<T> {
        unsafe { &*self.0.as_ptr().cast() }
    }

    #[inline(always)]
    pub fn as_uninit_mut(&mut self) -> &mut MaybeUninit<T> {
        unsafe { &mut *self.0.as_mut_ptr().cast() }
    }

    #[inline(always)]
    pub fn set(self, value: T) -> Init<M>
    where
        T: Sized,
    {
        let mut mem = self.0;
        unsafe {
            core::ptr::write(mem.as_mut_ptr(), value);
            Init::from_mem(mem)
        }
    }
}

impl<T, M> Uninit<M>
where
    M: Mem<Shape = [T]>,
{
    #[inline(always)]
    pub fn as_uninit_slice(&self) -> &[MaybeUninit<T>] {
        unsafe { &*(self.0.as_ptr() as *const _) }
    }

    #[inline(always)]
    pub fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe { &mut *(self.0.as_mut_ptr() as *mut _) }
    }
}

impl<M: Mem> From<M> for Uninit<M> {
    #[inline(always)]
    fn from(mem: M) -> Self {
        Self(mem)
    }
}

impl<'s, T> From<Slot<'s, T>> for Uninit<OwnedMem<'s, T>> {
    #[inline(always)]
    fn from(uninit: Slot<'s, T>) -> Self {
        Self(OwnedMem::from_slot(uninit))
    }
}
