#![warn(unsafe_op_in_unsafe_fn)]

mod private;
mod slice;
mod slot;
mod out;

pub mod scope;

pub use slice::{SliceLike, SliceSlot};
pub use slot::Slot;
pub use out::Out;

pub fn box_init<T, F>(init: F) -> Box<T>
where
    F: for<'s> FnOnce(Out<'s, T>) -> Slot<'s, T>,
{
    unsafe {
        let boxed = private::box_new_uninit_polyfill::<T>();
        let raw = Box::into_raw(boxed) as *mut T;

        scope::enter(|scope| {
            let slot = Out::new_unchecked(raw, scope);
            let slot = init(slot);
            Slot::leak(slot);
        });

       Box::from_raw(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let b: Box<i32> = box_init(|slot: Out<'_, i32>| -> Slot<'_, i32> {
            let filled: Slot<'_, i32> = slot.fill(50);
            let val = *filled;
            let slot: Out<'_, i32> = Slot::drop(filled);
            slot.fill(val * 2)
        });

        assert_eq!(*b, 100);
    }

    #[test]
    fn fibonacci() {
        let numbers: Box<[f64; 10_000]> = box_init(|slot| {
            let mut slice = SliceSlot::new(slot);
            loop {
                let done = slice.fill_next(|filled| match filled {
                    [.., a, b] => *a + *b,
                    _ => 1.0,
                });
                if done.is_err() {
                    return slice.finish().unwrap();
                }
            }
        });

        assert_eq!(numbers.last(), Some(&f64::INFINITY));
    }
}
