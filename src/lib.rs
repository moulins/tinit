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
    fn fibonacci1() {
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

    #[cfg(feature="enable_second_test")]
    #[test]
    fn fibonacci2() {
        let numbers: Box<[f64; 1_500]> = box_init(|slot| {
            let mut slice: SliceSlot<[f64; 1500]> = SliceSlot::new(slot);
            loop {
                // Inlining of SliceSlot::fill_next
                if slice.is_full() {
                    return Into::into(slice.finish().unwrap());
                }
        
                let filled_len = &mut slice.filled;
                let ptr = SliceLike::ptr_mut(&mut slice.slot);
        
                unsafe {
                    let filled = std::slice::from_raw_parts_mut(ptr, *filled_len);
                    let next = ptr.add(*filled_len);
                    crate::token::scope(|scope| {
                        let uninit = Out::forge(scope, next);

                        let val = match filled {
                            [.., a, b] => *a + *b,
                            _ => 1.0,
                        };
                        let _token: Token<'_> = uninit.fill(val).into();
                    });
                    *filled_len += 1;
                }
            }
        });

        assert_eq!(numbers.last(), Some(&f64::INFINITY));
    }
}
