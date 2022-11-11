use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::{ptr, slice};

use crate::init::Init;
use crate::place::{Place, SlicePlace};

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
    pub fn emplace(&mut self) -> SliceElemPlace<'_, P> {
        if self.is_full() {
            panic_slice_full(self.len())
        } else {
            SliceElemPlace(self)
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
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
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
    panic!("slice isn't full(len: {len}, capacity: {cap})")
}

pub struct SliceElemPlace<'a, P: SlicePlace>(&'a mut Slice<P>);

impl<'a, T, P> SliceElemPlace<'a, P>
where
    P: SlicePlace<Elem = T>,
{
    #[inline(always)]
    pub fn filled(&self) -> &[T] {
        self.0
    }

    #[inline(always)]
    pub fn filled_mut(&mut self) -> &mut [T] {
        self.0
    }

    #[inline(always)]
    pub fn pos(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    pub fn split(self) -> (&'a mut [T], impl Place<Type = T, Init = ()> + 'a) {
        let filled = unsafe { slice::from_raw_parts_mut(self.0.as_mut_ptr(), self.0.len()) };
        (filled, self)
    }
}

unsafe impl<'a, P: SlicePlace> Place for SliceElemPlace<'a, P> {
    type Type = P::Elem;

    type Init = ();

    #[inline(always)]
    fn as_ptr(&self) -> *const Self::Type {
        let ptr = self.0.place.as_ptr() as *const Self::Type;
        unsafe { ptr.add(self.0.len) }
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut Self::Type {
        let ptr = self.0.place.as_mut_ptr() as *mut Self::Type;
        unsafe { ptr.add(self.0.len) }
    }

    #[inline(always)]
    unsafe fn assume_init(self) -> Self::Init {
        self.0.len += 1;
    }
}
