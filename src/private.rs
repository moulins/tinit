use core::mem::MaybeUninit;

/// Private trait to seal other traits in this crate.
pub trait Sealed {}

// TODO: remove once raw slice's len method stabilizes.
#[inline]
pub unsafe fn raw_slice_len_polyfill<T>(ptr: *const [T]) -> usize {
    let zst_slice: &[()] = &*(ptr as *const _);
    zst_slice.len()
}

// TODO: remove once Box::new_uninit stabilizes.
#[inline]
pub fn box_new_uninit_polyfill<T>() -> Box<MaybeUninit<T>> {
    unsafe {
        let layout = std::alloc::Layout::new::<T>();
        let raw = if layout.size() == 0 {
            core::ptr::NonNull::<T>::dangling().as_ptr()
        } else {
            std::alloc::alloc(layout) as *mut T
        };
        Box::from_raw(raw as *mut MaybeUninit<T>)
    }
}
