use alloc::boxed::Box;
use core::mem::MaybeUninit;

#[inline]
pub fn box_new_uninit<T>() -> Box<MaybeUninit<T>> {
    unsafe {
        let layout = alloc::alloc::Layout::new::<T>();
        let raw = if layout.size() == 0 {
            core::ptr::NonNull::<T>::dangling().as_ptr()
        } else {
            alloc::alloc::alloc(layout) as *mut T
        };
        Box::from_raw(raw as *mut MaybeUninit<T>)
    }
}

// SAFETY: `b` must contain a valid `T`.
#[inline(always)]
pub unsafe fn box_assume_init<T>(b: Box<MaybeUninit<T>>) -> Box<T> {
    let ptr = Box::into_raw(b).cast();
    // SAFETY: From the function's requirements.
    unsafe { Box::from_raw(ptr) }
}
