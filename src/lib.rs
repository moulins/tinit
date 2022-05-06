#![warn(unsafe_op_in_unsafe_fn)]

mod impls;
mod out;
mod private;
mod slice;
mod slot;

pub mod token;

pub use out::Out;
pub use slice::{SliceLike, SliceSlot};
pub use slot::Slot;
pub use token::{Token, Tokens};

pub fn box_init<T, F>(init: F) -> Box<T>
where
    F: for<'s> FnOnce(Out<'s, T>) -> Token<'s>,
{
    unsafe {
        let boxed = private::box_new_uninit_polyfill::<T>();
        let raw = Box::into_raw(boxed) as *mut T;

        token::scope(|scope| {
            let slot = Out::forge(scope, raw);
            let _token = init(slot);
        });

       Box::from_raw(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let b: Box<i32> = box_init(|slot| {
            let filled = slot.fill(50);
            let val = *filled;
            let slot = Slot::drop(filled);
            slot.fill(val * 2).into()
        });

        assert_eq!(*b, 100);
    }

    #[test]
    fn fibonacci() {
        let numbers: Box<[f64; 1_500]> = box_init(|slot| {
            let mut slice = SliceSlot::new(slot);
            loop {
                let done = slice.fill_next(|filled| match filled {
                    [.., a, b] => *a + *b,
                    _ => 1.0,
                });
                if done.is_err() {
                    return Into::into(slice.finish().unwrap());
                }
            }
        });

        assert_eq!(numbers.last(), Some(&f64::INFINITY));
    }
}
