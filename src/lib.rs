mod slot;
mod uninit;

pub mod scope;

pub use slot::Slot;
pub use uninit::Uninit;

pub fn box_init<T, F>(init: F) -> Box<T>
where
    F: for<'s> FnOnce(Uninit<'s, T>) -> Slot<'s, T>,
{
    // TODO: Use Box::new_uninit once stable
    let layout = std::alloc::Layout::new::<T>();
    let raw = if layout.size() == 0 {
        std::ptr::NonNull::<T>::dangling().as_ptr()
    } else {
        unsafe { std::alloc::alloc(layout) as *mut T }
    };

    scope::enter(|scope| {
        let slot = unsafe { Uninit::new_unchecked(raw, scope) };
        let _own = init(slot);
    });

    unsafe { Box::from_raw(raw) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use super::*;
        let b: Box<i32> = box_init(|slot: Uninit<'_, i32>| -> Slot<'_, i32> {
            let filled: Slot<'_, i32> = slot.fill(50);
            let val = *filled;
            let slot: Uninit<'_, i32> = Slot::drop(filled);
            slot.fill(val * 2)
        });

        assert_eq!(*b, 100);
    }
}
