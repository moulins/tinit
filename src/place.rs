use core::mem::MaybeUninit;
use core::ptr::NonNull;

use crate::{Loan, Out};

pub trait IntoPlace: Sized {
    type Type: ?Sized;
    type Place: Place<Type = Self::Type, Init = Self::Init>;
    type Init;

    // Safety: forgetting or leaking the returned place is UB
    unsafe fn into_place(self) -> Self::Place;

    #[inline(always)]
    fn set(self, value: Self::Type) -> Self::Init
    where
        Self::Type: Sized,
    {
        unsafe {
            let mut place = self.into_place();
            core::ptr::write(place.as_mut_ptr(), value);
            place.assume_init()
        }
    }

    #[inline(always)]
    fn with(
        self,
        init: impl for<'s> FnOnce(Out<'s, Self::Type>) -> Loan<'s, Self::Type>,
    ) -> Self::Init {
        let mut place = unsafe { self.into_place() };
        {
            make_lease!(lease);
            let out = lease.borrow(&mut place);
            core::mem::forget(init(out));
        }
        unsafe { place.assume_init() }
    }
}

pub struct PlaceFn<F>(pub F);

impl<F, P> IntoPlace for PlaceFn<F>
where
    F: FnOnce() -> P,
    P: Place,
{
    type Type = P::Type;
    type Place = P;
    type Init = P::Init;

    #[inline(always)]
    unsafe fn into_place(self) -> Self::Place {
        (self.0)()
    }
}

// TODO: what about Send + Sync?
pub unsafe trait Place: Sized {
    type Type: ?Sized;
    type Init;

    fn as_ptr(&self) -> *const Self::Type;

    fn as_mut_ptr(&mut self) -> *mut Self::Type;

    // Safety: the place must be initialized.
    unsafe fn assume_init(self) -> Self::Init;

    #[inline(always)]
    fn as_non_null(&mut self) -> NonNull<Self::Type> {
        unsafe { NonNull::new_unchecked(self.as_mut_ptr()) }
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

impl<P: Place> IntoPlace for P {
    type Type = P::Type;
    type Place = P;
    type Init = P::Init;

    #[inline(always)]
    unsafe fn into_place(self) -> Self::Place {
        self
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
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut _) };
        <Self::Type as sealed::SliceLike>::len(ptr)
    }

    #[inline(always)]
    fn as_uninit_slice(&self) -> &[MaybeUninit<Self::Elem>] {
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut _) };
        let data = <Self::Type as sealed::SliceLike>::as_elem_ptr(ptr);
        let len = <Self::Type as sealed::SliceLike>::len(ptr);
        unsafe { core::slice::from_raw_parts(data.as_ptr() as *mut _, len) }
    }

    #[inline(always)]
    fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<Self::Elem>] {
        let ptr = self.as_non_null();
        let data = <Self::Type as sealed::SliceLike>::as_elem_ptr(ptr);
        let len = <Self::Type as sealed::SliceLike>::len(ptr);
        unsafe { core::slice::from_raw_parts_mut(data.as_ptr() as *mut _, len) }
    }
}

mod sealed {
    use core::ptr::NonNull;

    pub trait SliceLike {
        type Elem;
        fn len(ptr: NonNull<Self>) -> usize;
        fn as_elem_ptr(ptr: NonNull<Self>) -> NonNull<Self::Elem>;
    }

    impl<const N: usize, T> SliceLike for [T; N] {
        type Elem = T;

        #[inline(always)]
        fn len(_: NonNull<Self>) -> usize {
            N
        }

        #[inline(always)]
        fn as_elem_ptr(ptr: NonNull<Self>) -> NonNull<Self::Elem> {
            ptr.cast()
        }
    }

    impl<T> SliceLike for [T] {
        type Elem = T;

        #[inline(always)]
        fn len(ptr: NonNull<Self>) -> usize {
            ptr.len()
        }

        #[inline(always)]
        fn as_elem_ptr(ptr: NonNull<Self>) -> NonNull<Self::Elem> {
            ptr.cast()
        }
    }
}
