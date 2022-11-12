use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::{ptr, slice};

use crate::init::Init;
use crate::place::{IntoPlace, Place, SlicePlace};

// TODO: document
// TODO: implement the full Vec API.
pub struct Slice<P: SlicePlace> {
    place: P,
    len: usize,
}

impl<T, P> Slice<P>
where
    P: SlicePlace<Elem = T>,
{
    #[inline(always)]
    pub fn new(place: P) -> Self {
        Self { place, len: 0 }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.place.len()
    }

    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.len >= self.place.len()
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.place.as_ptr() as *const T
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.place.as_mut_ptr() as *mut T
    }

    #[inline]
    pub fn push(&mut self, elem: T) {
        self.emplace().set(elem)
    }

    #[inline]
    pub fn emplace(&mut self) -> SliceEmplace<'_, P> {
        let pos = self.len();
        if pos >= self.place.len() {
            panic_slice_full(pos)
        } else {
            SliceEmplace { slice: self, pos }
        }
    }

    #[inline]
    pub fn emplace_at(&mut self, pos: usize) -> SliceEmplace<'_, P> {
        let len = self.len();
        if len >= self.place.len() {
            panic_slice_full(len)
        } else if pos > len {
            panic!("index out of bounds");
        } else {
            SliceEmplace { slice: self, pos }
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

    // TODO: a variant of 'pop' that returns an Option<Own<'_, T>>

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        // SAFETY: the slice is now empty, so drop all elements.
        unsafe { ptr::drop_in_place(self.deref_mut()) }
    }

    #[inline(always)]
    pub fn drop(mut self) -> P {
        self.clear();
        self.forget()
    }

    #[inline(always)]
    pub fn forget(self) -> P {
        // Disable the drop impl.
        let this = ManuallyDrop::new(self);
        // SAFETY: `this` isn't accessed nor dropped after this line.
        unsafe { ptr::read(&this.place) }
    }

    #[inline(always)]
    pub fn leak<'a>(self) -> &'a mut [T]
    where
        P: 'a,
    {
        let len = self.len();
        let p = Self::forget(self).as_mut_ptr();

        // SAFETY: `place` contains a valid `[T]`, and the lifetime is properly constrained.
        unsafe { core::slice::from_raw_parts_mut(p.cast(), len) }
    }

    #[inline]
    pub fn assert_full(self) -> Init<P> {
        if self.is_full() {
            unsafe { Init::from_place(Self::forget(self)) }
        } else {
            panic_slice_not_full(self.len(), self.capacity())
        }
    }
}

impl<P: SlicePlace> Deref for Slice<P> {
    type Target = [P::Elem];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }
}

impl<P: SlicePlace> DerefMut for Slice<P> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let (len, ptr) = (self.len(), self.as_mut_ptr());
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }
}

impl<P: SlicePlace> Drop for Slice<P> {
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
    panic!("slice isn't full (len: {len}, capacity: {cap})")
}

pub struct SliceEmplace<'a, P: SlicePlace> {
    slice: &'a mut Slice<P>,
    pos: usize,
}

impl<'a, T, P> IntoPlace for SliceEmplace<'a, P>
where
    P: SlicePlace<Elem = T>,
{
    type Type = T;

    type Place = SliceElem<'a, T>;

    type Init = ();

    #[inline]
    unsafe fn into_place(self) -> Self::Place {
        let elem = unsafe { self.slice.as_mut_ptr().add(self.pos) };
        let len = &mut self.slice.len;
        let shift = *len - self.pos;
        unsafe { core::ptr::copy(elem, elem.add(1), shift) }
        SliceElem { elem, shift, len }
    }
}

impl<'a, T, P> SliceEmplace<'a, P>
where
    P: SlicePlace<Elem = T>,
{
    #[inline(always)]
    pub fn filled(&self) -> &[T] {
        self.slice
    }

    #[inline(always)]
    pub fn filled_mut(&mut self) -> &mut [T] {
        self.slice
    }

    #[inline(always)]
    pub fn pos(&self) -> usize {
        self.pos
    }
}

use private::SliceElem;
mod private {
    use super::*;

    pub struct SliceElem<'a, T> {
        pub(super) elem: *mut T,
        pub(super) shift: usize,
        pub(super) len: &'a mut usize,
    }

    unsafe impl<'a, T> Place for SliceElem<'a, T> {
        type Type = T;
        type Init = ();

        #[inline(always)]
        fn as_ptr(&self) -> *const Self::Type {
            self.elem
        }

        #[inline(always)]
        fn as_mut_ptr(&mut self) -> *mut Self::Type {
            self.elem
        }

        #[inline(always)]
        unsafe fn assume_init(self) -> Self::Init {
            *self.len += 1;
            core::mem::forget(self);
        }
    }

    impl<'a, T> Drop for SliceElem<'a, T> {
        #[inline(always)]
        fn drop(&mut self) {
            unsafe {
                core::ptr::copy(self.elem.add(1), self.elem, self.shift);
            }
        }
    }
}
