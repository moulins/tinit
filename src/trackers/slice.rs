use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::{slice, ptr};

use crate::mem::MemSlice;
use crate::{Loan, Out};

use super::{Uninit, Init};

// TODO: document
// TODO: implement the full Vec API.
pub struct Slice<M: MemSlice> {
    mem: M,
    len: usize,
}

impl<T, M> Slice<M>
where
    M: MemSlice<Elem = T>,
{
    #[inline(always)]
    pub fn new(uninit: Uninit<M>) -> Self {
        Self { mem: uninit.into_mem(), len: 0 }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.mem.len()
    }

    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.len >= self.mem.len()
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.mem.as_ptr() as *const T
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.mem.as_mut_ptr() as *mut T
    }

    #[inline]
    pub fn push(&mut self, elem: T) {
        if self.is_full() {
            panic_slice_full(self.len())
        } else {
            unsafe {
                let p = self.as_mut_ptr().add(self.len);
                ptr::write(p, elem);
                self.len += 1;
            }
        }
    }

    #[inline]
    pub fn push_with<F>(&mut self, init: F)
    where
        F: for<'s> FnOnce(&mut [T], Out<'s, T>) -> Loan<'s, T>,
    {
        if self.is_full() {
            panic_slice_full(self.len())
        } else {
            make_lease!(lease);
            let elem = unsafe {
                let p = self.as_mut_ptr().add(self.len);
                lease.borrow_ptr(p)
            };

            let slice = DerefMut::deref_mut(self);
            Loan::forget(init(slice, elem.into()));

            self.len += 1;
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            let val = unsafe {
                self.len -= 1;
                ptr::read(self.as_mut_ptr().add(self.len))
            };
            Some(val)
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        // SAFETY: the slice is now empty, so drop all elements.
        unsafe { ptr::drop_in_place(self.deref_mut()) }
    }

    #[inline(always)]
    fn into_mem(this: Self) -> M {
        // Disable the drop impl.
        let this = ManuallyDrop::new(this);
        // SAFETY: `this` isn't accessed nor dropped after this line.
        unsafe { ptr::read(&this.mem) }
    }

    #[inline(always)]
    pub fn drop(mut self) -> Uninit<M> {
        self.clear();
        Self::into_mem(self).into()
    }

    #[inline(always)]
    pub fn forget(self) -> Uninit<M> {
        Self::into_mem(self).into()
    }

    #[inline(always)]
    pub fn leak<'a>(self) -> &'a mut [T] where M: 'a {
        let len = self.len();
        let p = Self::into_mem(self).as_mut_ptr();
        
        // SAFETY: `mem` contains a valid `[T]`, and the lifetime is properly constrained.
        unsafe { core::slice::from_raw_parts_mut(p.cast(), len) } 
    }

    #[inline]
    pub fn assert_full(self) -> Init<M> {
        if self.is_full() {
            unsafe { Init::from_mem(Self::into_mem(self)) }
        } else {
            panic_slice_not_full(self.len(), self.capacity())
        }
    }

    // TODO: add forget/leak/etc methods
}

impl<M: MemSlice> Deref for Slice<M> {
    type Target = [M::Elem];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }
}

impl<M: MemSlice> DerefMut for Slice<M> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }
}

impl<M: MemSlice> Drop for Slice<M> {
    #[inline(always)]
    fn drop(&mut self) {
        self.clear();
    }
}

#[cold]
#[inline(never)]
fn panic_slice_full(len: usize) -> ! {
    panic!("slice is already full (len: {len})")
}

#[cold]
#[inline(never)]
fn panic_slice_not_full(len: usize, cap: usize) -> ! {
    panic!("slice isn't full(len: {len}, capacity: {cap})")
}
