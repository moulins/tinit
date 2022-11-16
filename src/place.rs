use core::mem::{ManuallyDrop, MaybeUninit};

use crate::uninit::{UninitMut, UninitRef};
use crate::{ScopedMem, ScopedRef};

pub trait IntoPlace: Sized {
    type Type: ?Sized;
    type Place: Place<Type = Self::Type, Init = Self::Init>;
    type Init;

    // Safety: forgetting or leaking the returned place is UB
    unsafe fn materialize(self) -> Self::Place;

    #[inline(always)]
    fn set(self, value: Self::Type) -> Self::Init
    where
        Self::Type: Sized,
    {
        let mut place = unsafe { self.materialize() };
        place.raw_mut().write(value);
        unsafe { place.finalize() }
    }

    #[inline(always)]
    fn with(
        self,
        init: impl for<'s> FnOnce(ScopedMem<'s, Self::Type>) -> ScopedRef<'s, Self::Type>,
    ) -> Self::Init {
        let mut place = unsafe { self.materialize() };
        {
            make_scope!(scope);
            let out = scope.borrow(&mut place);
            core::mem::forget(init(out));
        }
        unsafe { place.finalize() }
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
    unsafe fn materialize(self) -> Self::Place {
        (self.0)()
    }
}

// TODO: what about Send + Sync?
pub unsafe trait Place: Sized {
    type Type: ?Sized;
    type Init;

    fn raw_ref(&self) -> UninitRef<'_, Self::Type>;

    fn raw_mut(&mut self) -> UninitMut<'_, Self::Type>;

    // Safety: the place must be initialized.
    unsafe fn finalize(self) -> Self::Init;

    // TODO: should this be unsafe? technically no, as creating a Place
    // is already unsafe in the general case.
    #[inline(always)]
    fn leak<'a>(self) -> UninitMut<'a, Self::Type>
    where
        Self: 'a,
    {
        // Disable the drop impl of the place.
        let mut this = ManuallyDrop::new(self);
        // SAFETY: the place has been 'disabled', so we can soundly extend the lifetime.
        unsafe { this.raw_mut().transmute_lt() }
    }
}

impl<P: Place> IntoPlace for P {
    type Type = P::Type;
    type Place = P;
    type Init = P::Init;

    #[inline(always)]
    unsafe fn materialize(self) -> Self::Place {
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
        <Self::Type as sealed::SliceLike>::len(self.raw_ref().into())
    }

    #[inline(always)]
    fn as_uninit_slice(&self) -> &[MaybeUninit<Self::Elem>] {
        let ptr = self.raw_ref().into();
        let len = <Self::Type as sealed::SliceLike>::len(ptr);
        unsafe { core::slice::from_raw_parts(ptr.cast().as_ptr(), len) }
    }

    #[inline(always)]
    fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<Self::Elem>] {
        let ptr = self.raw_ref().into();
        let len = <Self::Type as sealed::SliceLike>::len(ptr);
        unsafe { core::slice::from_raw_parts_mut(ptr.cast().as_ptr(), len) }
    }
}

mod sealed {
    use core::ptr::NonNull;

    pub trait SliceLike {
        type Elem;
        fn len(ptr: NonNull<Self>) -> usize;
    }

    impl<const N: usize, T> SliceLike for [T; N] {
        type Elem = T;

        #[inline(always)]
        fn len(_: NonNull<Self>) -> usize {
            N
        }
    }

    impl<T> SliceLike for [T] {
        type Elem = T;

        #[inline(always)]
        fn len(ptr: NonNull<Self>) -> usize {
            ptr.len()
        }
    }
}
