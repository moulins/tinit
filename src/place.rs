use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::Deref;

use crate::uninit::{UninitMut, UninitRef};
use crate::{ScopedMem, ScopedRef};

pub trait Slot<V: Deref>: Sized {
    type Place: Place<Target = V::Target, Init = V>;

    fn into_place(self) -> Self::Place;

    #[inline(always)]
    fn set(self, value: V::Target) -> V
    where
        V::Target: Sized,
    {
        let mut place = self.into_place();
        place.deref_place_mut().write(value);
        unsafe { place.assume_init() }
    }

    #[inline(always)]
    fn with(
        self,
        init: impl for<'s> FnOnce(ScopedMem<'s, V::Target>) -> ScopedRef<'s, V::Target>,
    ) -> V {
        let mut place = self.into_place();
        {
            let_scope!(scope);
            let out = scope.borrow(&mut place);
            core::mem::forget(init(out));
        }
        unsafe { place.assume_init() }
    }
}

impl<P: Place> Slot<P::Init> for P {
    type Place = P;

    #[inline(always)]
    fn into_place(self) -> Self::Place {
        self
    }
}

pub unsafe trait Place: Sized {
    // Technically could be deduced from Self::Init, but makes the trait easier to use.
    type Target: ?Sized;
    type Init: Deref<Target = Self::Target>;

    fn deref_place(&self) -> UninitRef<'_, Self::Target>;

    fn deref_place_mut(&mut self) -> UninitMut<'_, Self::Target>;

    // Safety: the place must be initialized.
    unsafe fn assume_init(self) -> Self::Init;

    // TODO: should this be unsafe? technically no, as creating a Place
    // that *must* be dropped requires unsafe code.
    #[inline(always)]
    fn leak<'a>(self) -> UninitMut<'a, Self::Target>
    where
        Self: 'a,
    {
        // Disable the drop impl of the place.
        let mut this = ManuallyDrop::new(self);
        // SAFETY: the place has been 'disabled', so we can soundly extend the lifetime.
        unsafe { this.deref_place_mut().transmute_lt() }
    }
}

pub trait Emplace: Deref {
    type Place: Place<Target = Self::Target, Init = Self>;

    fn emplace() -> Self::Place;
}

/// A slice-like place. Implemented for places to slices and fixed-sized arrays.
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
    P::Target: sealed::SliceLike,
{
    type Elem = <Self::Target as sealed::SliceLike>::Elem;

    #[inline(always)]
    fn len(&self) -> usize {
        <Self::Target as sealed::SliceLike>::len(self.deref_place().into())
    }

    #[inline(always)]
    fn as_uninit_slice(&self) -> &[MaybeUninit<Self::Elem>] {
        let ptr = self.deref_place().into();
        let len = <Self::Target as sealed::SliceLike>::len(ptr);
        unsafe { core::slice::from_raw_parts(ptr.cast().as_ptr(), len) }
    }

    #[inline(always)]
    fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<Self::Elem>] {
        let ptr = self.deref_place().into();
        let len = <Self::Target as sealed::SliceLike>::len(ptr);
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
