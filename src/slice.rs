use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::{ptr, slice};

use crate::init::Init;
use crate::place::{Place, SlicePlace, Slot};

// TODO: document
// TODO: implement the full Vec API.
pub struct Slice<P: SlicePlace> {
    place: P,
    len: usize,
    // We logically own the value stored in the place.
    _marker: PhantomData<P::Init>,
}

impl<T, P> Slice<P>
where
    P: SlicePlace<Elem = T>,
{
    #[inline(always)]
    pub fn new(place: P) -> Self {
        Self {
            place,
            len: 0,
            _marker: PhantomData,
        }
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
        self.place.deref_place().as_ptr().cast()
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.place.deref_place_mut().as_mut_ptr().cast()
    }

    #[inline]
    pub fn push(&mut self, elem: T) {
        self.emplace().set(elem);
    }

    #[inline]
    pub fn emplace(&mut self) -> SliceHole<'_, T> {
        let pos = self.len();
        if pos >= self.place.len() {
            panic_slice_full(pos)
        } else {
            unsafe { SliceHole::open(self, pos) }
        }
    }

    #[inline]
    pub fn emplace_at(&mut self, pos: usize) -> SliceHole<'_, T> {
        let len = self.len();
        if len >= self.place.len() {
            panic_slice_full(len)
        } else if pos > len {
            panic!("index out of bounds");
        } else {
            unsafe { SliceHole::open(self, pos) }
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
        let len = core::mem::replace(&mut self.len, 0);
        // SAFETY: the slice is now empty, so drop all elements.
        unsafe { 
            let slice = ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), len);
            ptr::drop_in_place(slice);
        }
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

    // TODO: see Place::leak
    #[inline(always)]
    pub fn leak<'a>(self) -> &'a mut [T]
    where
        P: 'a,
    {
        let len = self.len();
        let p = Self::forget(self).leak().as_mut_ptr();
        // SAFETY: `place` contains a valid `[T]`, and the lifetime is properly constrained.
        unsafe { core::slice::from_raw_parts_mut(p.cast(), len) }
    }

    #[inline]
    pub fn assert_full(self) -> Init<P> {
        if self.is_full() {
            unsafe { Init::new_unchecked(Self::forget(self)) }
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

pub struct SliceHole<'a, T> {
    len: &'a mut usize,
    prefix: &'a mut [T],
    // Only the first element is uninitialized
    suffix: &'a mut [MaybeUninit<T>],
}

impl<'a, T> SliceHole<'a, T> {
    // SAFETY: `0..=slice.len().contains(pos) && !slice.is_full()`
    unsafe fn open<P: SlicePlace<Elem=T>>(slice: &'a mut Slice<P>, pos: usize) -> Self {
        // 'Pre-poop our pants' so that leaking this place leaks all moved elements.
        let len = &mut slice.len;
        let suffix_len = core::mem::replace(len, pos) - pos;
        let ptr = slice.place.deref_place_mut().as_mut_ptr() as *mut T;

        Self {
            len,
            prefix: unsafe { slice::from_raw_parts_mut(ptr, pos) },
            suffix: unsafe {
                // Move suffix one element to the right to open the hole
                let ptr = ptr.add(pos);
                ptr::copy(ptr, ptr.add(1), suffix_len);
                slice::from_raw_parts_mut(ptr.cast(), suffix_len + 1)
            },
        }
    }

    #[inline(always)]
    pub fn pos(&self) -> usize {
        self.prefix.len()
    }

    #[inline(always)]
    pub fn split_ref(&self) -> (&[T], &[T]) {
        let suffix = unsafe { &*(self.suffix.get_unchecked(1..) as *const _ as *const [T]) };
        (self.prefix, suffix)
    }

    #[inline(always)]
    pub fn split_mut(&mut self) -> (&mut [T], &mut [T]) {
        let suffix = unsafe { &mut *(self.suffix.get_unchecked_mut(1..) as *mut _ as *mut [T]) };
        (self.prefix, suffix)
    }

    #[inline(always)]
    pub fn split_prefix(mut self) -> (&'a mut [T], impl Place<Init=&'a mut T>) {
        (core::mem::take(&mut self.prefix), self)
    }
}

unsafe impl<'a, T: 'a> Place for SliceHole<'a, T> {
    type Target = T;
    type Init = &'a mut T;

    #[inline(always)]
    fn deref_place(&self) -> crate::uninit::UninitRef<'_, Self::Target> {
        unsafe { self.suffix.get_unchecked(0).into() }
    }

    #[inline(always)]
    fn deref_place_mut(&mut self) -> crate::uninit::UninitMut<'_, Self::Target> {
        unsafe { self.suffix.get_unchecked_mut(0).into() }
    }

    #[inline(always)]
    unsafe fn assume_init(self) -> Self::Init {
        // Element is initialized, put back correct length.
        *self.len += self.suffix.len();
        unsafe {
            // Disable drop impl and return a reference to the initialized element.
            let mut this = ManuallyDrop::new(self);
            &mut *(this.suffix.as_mut_ptr() as *mut T)
        }
    }
}

impl<'a, T> Drop for SliceHole<'a, T> {
    #[inline(always)]
    fn drop(&mut self) {
        // Shift back suffix and fix the length
        let n = self.suffix.len() - 1;
        let ptr = self.suffix.as_mut_ptr();
        unsafe {
            ptr::copy(ptr.add(1), ptr, n);
            *self.len += n;
        }
    }
}
