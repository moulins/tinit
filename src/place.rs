use core::mem::MaybeUninit;

use crate::{Loan, Out};

pub unsafe trait Place: Sized {
    type Type: ?Sized;
    type Init;

    fn as_ptr(&self) -> *const Self::Type;

    fn as_mut_ptr(&mut self) -> *mut Self::Type;

    unsafe fn assume_init(self) -> Self::Init;

    #[inline(always)]
    fn set(mut self, value: Self::Type) -> Self::Init
    where
        Self::Type: Sized,
    {
        unsafe {
            core::ptr::write(self.as_mut_ptr(), value);
            self.assume_init()
        }
    }

    #[inline(always)]
    fn with(
        mut self,
        init: impl for<'s> FnOnce(Out<'s, Self::Type>) -> Loan<'s, Self::Type>,
    ) -> Self::Init {
        {
            make_lease!(lease);
            let out = lease.borrow(&mut self);
            core::mem::forget(init(out));
        }
        unsafe { self.assume_init() }
    }

    #[inline(always)]
    fn as_uninit(&self) -> &MaybeUninit<Self::Type>
    where
        Self::Type: Sized,
    {
        unsafe { &*(self.as_ptr() as *const _) }
    }

    #[inline(always)]
    fn as_uninit_mut(&mut self) -> &mut MaybeUninit<Self::Type>
    where
        Self::Type: Sized,
    {
        unsafe { &mut *(self.as_mut_ptr() as *mut _) }
    }
}

/// A slice-like place. Implemented for slices and fixed-sized arrays.
// TODO: document more
pub unsafe trait SlicePlace: Place {
    type Elem;

    fn len(&self) -> usize;

    fn as_uninit_slice(&self) -> &[MaybeUninit<Self::Elem>];

    fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<Self::Elem>];
}

unsafe impl<P> SlicePlace for P
where
    P: Place,
    P::Type: sealed::SliceLike,
{
    type Elem = <Self::Type as sealed::SliceLike>::Elem;

    #[inline(always)]
    fn len(&self) -> usize {
        <Self::Type as sealed::SliceLike>::len(self.as_ptr())
    }

    #[inline(always)]
    fn as_uninit_slice(&self) -> &[MaybeUninit<Self::Elem>] {
        let ptr = self.as_ptr();
        let data = <Self::Type as sealed::SliceLike>::as_elem_ptr(ptr);
        let len = <Self::Type as sealed::SliceLike>::len(ptr);
        unsafe { core::slice::from_raw_parts(data as *const _, len) }
    }

    #[inline(always)]
    fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<Self::Elem>] {
        let ptr = self.as_mut_ptr() as *const _;
        let data = <Self::Type as sealed::SliceLike>::as_elem_ptr(ptr) as *mut Self::Elem;
        let len = <Self::Type as sealed::SliceLike>::len(ptr);
        unsafe { core::slice::from_raw_parts_mut(data as *mut _, len) }
    }
}

mod sealed {
    pub trait SliceLike {
        type Elem;
        fn len(ptr: *const Self) -> usize;
        fn as_elem_ptr(ptr: *const Self) -> *const Self::Elem;
    }

    impl<const N: usize, T> SliceLike for [T; N] {
        type Elem = T;

        #[inline(always)]
        fn len(_: *const Self) -> usize {
            N
        }

        #[inline(always)]
        fn as_elem_ptr(ptr: *const Self) -> *const Self::Elem {
            ptr as *const _
        }
    }

    impl<T> SliceLike for [T] {
        type Elem = T;

        #[inline(always)]
        fn len(ptr: *const Self) -> usize {
            // SAFETY: this is only called by `SlicePlace::len`, which ensures
            // that `ptr` points to a valid allocation.
            unsafe { crate::polyfill::raw_slice_len(ptr) }
        }

        #[inline(always)]
        fn as_elem_ptr(ptr: *const Self) -> *const Self::Elem {
            ptr as *const _
        }
    }
}
