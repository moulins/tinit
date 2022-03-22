use core::mem::{ManuallyDrop, MaybeUninit};
use core::{fmt, ptr, slice};

use crate::private::Sealed;
use crate::slot::Slot;
use crate::uninit::Uninit;

/// Trait for unifying behavior over slices and fixed-size arrays].
pub trait SliceLike: Sealed {
    type Elem;

    fn len(slot: &Uninit<'_, Self>) -> usize;
    fn ptr(slot: &Uninit<'_, Self>) -> *const Self::Elem;
    fn ptr_mut(slot: &mut Uninit<'_, Self>) -> *mut Self::Elem;
}

/// An owning handle to a partially initialized slice.
// TODO: document methods and safety invariants
// TODO: improve API
pub struct SliceSlot<'s, S: SliceLike> {
    slot: Uninit<'s, S>,
    filled: usize,
}

impl<'s, S: SliceLike> SliceSlot<'s, S> {
    #[inline]
    pub fn new(slot: Uninit<'s, S>) -> Self {
        Self { slot, filled: 0 }
    }

    #[inline]
    pub fn len(&self) -> usize {
        SliceLike::len(&self.slot)
    }

    #[inline]
    pub fn split(&self) -> (&[S::Elem], &[MaybeUninit<S::Elem>]) {
        unsafe {
            let filled_len = self.filled;
            let unfilled_len = SliceLike::len(&self.slot) - filled_len;
            let filled = SliceLike::ptr(&self.slot);
            let unfilled = filled.add(filled_len) as *const MaybeUninit<S::Elem>;
            (
                slice::from_raw_parts(filled, filled_len),
                slice::from_raw_parts(unfilled, unfilled_len),
            )
        }
    }

    #[inline]
    pub fn split_mut(&mut self) -> (&mut [S::Elem], &mut [MaybeUninit<S::Elem>]) {
        unsafe {
            let filled_len = self.filled;
            let unfilled_len = SliceLike::len(&self.slot) - filled_len;
            let filled = SliceLike::ptr_mut(&mut self.slot);
            let unfilled = filled.add(filled_len) as *mut MaybeUninit<S::Elem>;
            (
                slice::from_raw_parts_mut(filled, filled_len),
                slice::from_raw_parts_mut(unfilled, unfilled_len),
            )
        }
    }

    #[inline]
    pub fn filled(&self) -> &[S::Elem] {
        self.split().0
    }

    #[inline]
    pub fn unfilled(&self) -> &[MaybeUninit<S::Elem>] {
        self.split().1
    }

    #[inline]
    pub fn filled_mut(&mut self) -> &mut [S::Elem] {
        self.split_mut().0
    }

    #[inline]
    pub fn unfilled_mut(&mut self) -> &mut [MaybeUninit<S::Elem>] {
        self.split_mut().1
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.filled == self.len()
    }

    #[inline]
    pub unsafe fn assume_init(self) -> Slot<'s, S> {
        let this = ManuallyDrop::new(self);
        ptr::read(&this.slot).assume_init()
    }

    #[inline]
    pub fn finish(self) -> Result<Slot<'s, S>, Self> {
        if self.is_full() {
            unsafe { Ok(self.assume_init()) }
        } else {
            Err(self)
        }
    }

    pub fn fill_next<F>(&mut self, f: F) -> Result<&[S::Elem], ()>
    where
        F: FnOnce(&mut [S::Elem]) -> S::Elem,
    {
        self.init_next(|filled, slot| slot.fill(f(filled)))
    }

    pub fn init_next<'a, F>(&'a mut self, init: F) -> Result<&[S::Elem], ()>
    where
        F: for<'t> FnOnce(&'a mut [S::Elem], Uninit<'t, S::Elem>) -> Slot<'t, S::Elem>,
    {
        if self.is_full() {
            return Err(());
        }

        let filled_len = &mut self.filled;
        let ptr = SliceLike::ptr_mut(&mut self.slot);

        unsafe {
            let filled = slice::from_raw_parts_mut(ptr, *filled_len);
            let next = ptr.add(*filled_len);
            crate::scope::enter(|scope| {
                let uninit = Uninit::new_unchecked(next, scope);
                let _slot = init(filled, uninit);
            });
            *filled_len += 1;
        }
        Ok(self.filled())
    }

    pub fn fill_remaining<F>(mut self, mut f: F) -> Slot<'s, S>
    where
        F: for<'t> FnMut(&'t mut [S::Elem]) -> S::Elem,
    {
        while self.fill_next(|filled| f(filled)).is_ok() {
            // do nothing
        }
        unsafe { self.assume_init() }
    }

    pub fn init_remaining<F>(mut self, mut init: F) -> Slot<'s, S>
    where
        F: for<'t> FnMut(Uninit<'t, S::Elem>) -> Slot<'t, S::Elem>,
    {
        while self.init_next(|_, slot| init(slot)).is_ok() {
            // do nothing
        }
        unsafe { self.assume_init() }
    }
}

impl<'s, S: SliceLike> fmt::Debug for SliceSlot<'s, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SliceSlot")
            .field("len", &SliceLike::len(&self.slot))
            .field("filled", &self.filled)
            .finish_non_exhaustive()
    }
}

impl<'s, S: SliceLike> Drop for SliceSlot<'s, S> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.filled_mut()) }
    }
}

impl<'s, S: SliceLike> From<Uninit<'s, S>> for SliceSlot<'s, S> {
    #[inline]
    fn from(slot: Uninit<'s, S>) -> Self {
        Self::new(slot)
    }
}

impl<T> Sealed for [T] {}
impl<T> SliceLike for [T] {
    type Elem = T;

    fn len(slot: &Uninit<'_, Self>) -> usize {
        unsafe { crate::private::raw_slice_len_polyfill(slot.as_ptr()) }
    }

    fn ptr(slot: &Uninit<'_, Self>) -> *const Self::Elem {
        slot.as_ptr() as *const _
    }

    fn ptr_mut(slot: &mut Uninit<'_, Self>) -> *mut Self::Elem {
        slot.as_mut_ptr() as *mut _
    }
}

impl<T, const N: usize> Sealed for [T; N] {}
impl<T, const N: usize> SliceLike for [T; N] {
    type Elem = T;

    fn len(_slot: &Uninit<'_, Self>) -> usize {
        N
    }

    fn ptr(slot: &Uninit<'_, Self>) -> *const Self::Elem {
        slot.as_ptr() as *const _
    }

    fn ptr_mut(slot: &mut Uninit<'_, Self>) -> *mut Self::Elem {
        slot.as_mut_ptr() as *mut _
    }
}
