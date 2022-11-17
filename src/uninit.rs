//! Alternative to [`&'a {mut} MaybeUninit<T>`](MaybeUninit) with support for unsized types.

// In this module, unsafe ops trivially fulfill their precondition
// from the enclosing method.
#![allow(unsafe_op_in_unsafe_fn)]

use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr::{self, NonNull};

macro_rules! make {
    ($name:ident; $ptr:expr) => {
        $name {
            ptr: $ptr,
            _marker: PhantomData,
        }
    };
}

// TODO: document
#[repr(transparent)]
pub struct UninitRef<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a T>,
}

// UninitRef behaves like `&T`
unsafe impl<T: Sync + ?Sized> Sync for UninitRef<'_, T> {}
unsafe impl<T: Sync + ?Sized> Send for UninitRef<'_, T> {}
impl<T: ?Sized> Copy for UninitRef<'_, T> {}
impl<T: ?Sized> Clone for UninitRef<'_, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

// TODO: document
#[repr(transparent)]
pub struct UninitMut<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a mut T>,
}

// UninitMut behaves like `&mut T`
unsafe impl<T: Sync + ?Sized> Sync for UninitMut<'_, T> {}
unsafe impl<T: Sync + ?Sized> Send for UninitMut<'_, T> {}

impl<'a, T: ?Sized> UninitRef<'a, T> {
    // SAFETY: ptr must be live and immutably borrowed during `'a`.
    #[inline(always)]
    pub unsafe fn new_unchecked(ptr: *const T) -> Self {
        make!(Self; NonNull::new_unchecked(ptr as *mut _))
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub fn as_non_null(&self) -> NonNull<T> {
        self.ptr
    }

    #[inline(always)]
    pub unsafe fn as_ref(&self) -> &T {
        self.ptr.as_ref()
    }

    #[inline(always)]
    pub unsafe fn into_ref(self) -> &'a T {
        self.ptr.as_ref()
    }

    #[inline(always)]
    pub unsafe fn transmute_lt<'b>(self) -> UninitRef<'b, T> {
        make!(UninitRef; self.ptr)
    }
}

impl<'a, T: Sized> UninitRef<'a, T> {
    #[inline(always)]
    pub fn as_uninit(&self) -> &MaybeUninit<T> {
        unsafe { self.ptr.cast().as_ref() }
    }

    #[inline(always)]
    pub fn into_uninit(self) -> &'a MaybeUninit<T> {
        unsafe { self.ptr.cast().as_ref() }
    }

    #[inline(always)]
    pub unsafe fn read(self) -> T {
        ptr::read(self.ptr.as_ptr() as *const T)
    }
}

impl<'a, T: Sized> From<&'a MaybeUninit<T>> for UninitRef<'a, T> {
    #[inline(always)]
    fn from(uninit: &'a MaybeUninit<T>) -> Self {
        make!(Self; NonNull::from(uninit).cast())
    }
}

impl<'a, T: Sized> From<UninitRef<'a, T>> for &'a MaybeUninit<T> {
    #[inline(always)]
    fn from(uninit: UninitRef<'a, T>) -> Self {
        unsafe { uninit.ptr.cast().as_mut() }
    }
}

impl<T: ?Sized> From<UninitRef<'_, T>> for NonNull<T> {
    #[inline(always)]
    fn from(uninit: UninitRef<'_, T>) -> Self {
        uninit.ptr
    }
}

impl<T: ?Sized> fmt::Debug for UninitRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(core::any::type_name::<Self>())
    }
}

impl<'a, T: ?Sized> UninitMut<'a, T> {
    // SAFETY: ptr must be live and unaliased during `'a`.
    #[inline(always)]
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        make!(Self; NonNull::new_unchecked(ptr))
    }

    #[inline(always)]
    pub fn borrow(&self) -> UninitRef<'_, T> {
        make!(UninitRef; self.ptr)
    }

    #[inline(always)]
    pub fn borrow_mut(&mut self) -> UninitMut<'_, T> {
        make!(UninitMut; self.ptr)
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub fn as_non_null(&mut self) -> NonNull<T> {
        self.ptr
    }

    #[inline(always)]
    pub unsafe fn as_ref(&self) -> &T {
        self.ptr.as_ref()
    }

    #[inline(always)]
    pub unsafe fn as_mut(&mut self) -> &mut T {
        self.ptr.as_mut()
    }

    #[inline(always)]
    pub unsafe fn into_mut(mut self) -> &'a mut T {
        self.ptr.as_mut()
    }

    #[inline(always)]
    pub unsafe fn transmute_lt<'b>(self) -> UninitMut<'b, T> {
        make!(UninitMut; self.ptr)
    }

    #[inline(always)]
    pub unsafe fn drop_in_place(&mut self) {
        ptr::drop_in_place(self.ptr.as_ptr())
    }
}

impl<'a, T: Sized> UninitMut<'a, T> {
    #[inline(always)]
    pub fn as_uninit(&mut self) -> &mut MaybeUninit<T> {
        unsafe { self.ptr.cast().as_mut() }
    }

    #[inline(always)]
    pub fn into_uninit(self) -> &'a mut MaybeUninit<T> {
        unsafe { self.ptr.cast().as_mut() }
    }

    #[inline(always)]
    pub unsafe fn read(&self) -> T {
        ptr::read(self.ptr.as_ptr() as *const T)
    }

    #[inline(always)]
    pub fn write(&mut self, value: T) -> &mut T {
        unsafe {
            let ptr = self.ptr.as_ptr();
            ptr::write(ptr, value);
            &mut *ptr
        }
    }
}

impl<'a, T: Sized> From<&'a mut MaybeUninit<T>> for UninitMut<'a, T> {
    #[inline(always)]
    fn from(uninit: &'a mut MaybeUninit<T>) -> Self {
        make!(Self; NonNull::from(uninit).cast())
    }
}

impl<'a, T: Sized> From<UninitMut<'a, T>> for &'a mut MaybeUninit<T> {
    #[inline(always)]
    fn from(uninit: UninitMut<'a, T>) -> Self {
        unsafe { uninit.ptr.cast().as_mut() }
    }
}

impl<T: ?Sized> From<UninitMut<'_, T>> for NonNull<T> {
    #[inline(always)]
    fn from(uninit: UninitMut<'_, T>) -> Self {
        uninit.ptr
    }
}

impl<T: ?Sized> fmt::Debug for UninitMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(core::any::type_name::<Self>())
    }
}

// TODO: methods on Uninit{Mut, Ref}<[T]>?
// TODO: Unsize polyfill to replace ad-hoc SliceLike trait?
// TODO: MetaSized trait to get the layout of a Uninit{Ref, Mut} when possible?
