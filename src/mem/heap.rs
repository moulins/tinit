use core::mem::MaybeUninit;
use core::ptr::NonNull;
use core::{mem, slice};

use alloc::alloc::Layout;
use alloc::boxed::Box;

use crate::place::{Emplace, Place, Slot};
use super::Mem;

pub struct BoxMem<T: ?Sized>(Mem<'static, T>);

impl<T> BoxMem<T> {
    #[inline]
    pub fn alloc() -> Self {
        unsafe {
            let raw = if mem::size_of::<T>() == 0 {
                NonNull::<T>::dangling().as_ptr()
            } else {
                let layout = Layout::new::<T>();
                alloc::alloc::alloc(layout) as *mut T
            };
            Self(Mem::from_raw(raw))
        }
    }
}

impl<T> BoxMem<[T]> {
    #[inline]
    pub fn alloc_slice(len: usize) -> Self {
        let layout = Layout::from_size_align(
            mem::size_of::<T>().saturating_mul(len),
            mem::align_of::<T>(),
        ).unwrap_or_else(|_| panic!("slice capacity overflow"));

        unsafe {
            let raw = if layout.size() == 0 {
                NonNull::<T>::dangling().as_ptr()
            } else {
                alloc::alloc::alloc(layout) as *mut T
            };
            let slice = slice::from_raw_parts_mut(raw, len);
            Self(Mem::from_raw(slice))
        }
    }
}

unsafe impl<T> Place for BoxMem<T> {
    type Target = T;
    type Init = Box<T>;

    impl_place_deref!(use 0);

    #[inline(always)]
    unsafe fn assume_init(self) -> Self::Init {
        unsafe { Box::from_raw(self.0.into_raw()) }
    }
}

impl<T> Emplace for Box<T> {
    type Place = BoxMem<T>;

    #[inline]
    fn emplace() -> Self::Place {
        BoxMem::alloc()
    }
}

impl<T> Slot<Box<T>> for Box<MaybeUninit<T>> {
    type Place = BoxMem<T>;

    #[inline(always)]
    fn into_place(self) -> Self::Place {
        let ptr = Box::into_raw(self).cast();
        unsafe { BoxMem(Mem::from_raw(ptr)) }
    }
}

// TODO: impls for Box<[T]>
