#![no_std]
#![warn(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::boxed::Box;
use core::mem::MaybeUninit;

mod private;

/// Macro definitions and utilities.
#[doc(hidden)]
#[macro_use]
pub mod __;

pub mod mem;
mod polyfill;
pub mod trackers;

pub type Slot<'s, T> = &'s mut MaybeUninit<T>;
pub type Place<'s, T> = trackers::Uninit<mem::OwnedMem<'s, T>>;
pub type Own<'s, T> = trackers::Init<mem::OwnedMem<'s, T>>;
pub type Out<'s, T> = trackers::Uninit<mem::LeasedMem<'s, T>>;
pub type Loan<'s, T> = trackers::Init<mem::LeasedMem<'s, T>>;

// TODO: constify everything that can be constified.

pub fn box_init<T, F>(init: F) -> Box<T>
where
    F: for<'s> FnOnce(Out<'s, T>) -> Loan<'s, T>,
{
    let mut boxed: Box<MaybeUninit<T>> = polyfill::box_new_uninit();

    {
        make_lease!(lease);
        let mem = lease.borrow_slot(&mut boxed);
        Loan::forget(init(mem.into()));
    }

    // SAFETY: we got a `Loan` pointing to the box, so we know it's initialized.
    unsafe { polyfill::box_assume_init(boxed) }
}

#[inline(always)]
pub fn stack_slot<T>() -> MaybeUninit<T> {
    MaybeUninit::uninit()
}

#[cfg(test)]
mod tests {
    use super::*;
    use trackers::Slice;

    #[test]
    fn it_works() {
        let b: Box<i32> = box_init(|out: Out<'_, i32>| -> Loan<'_, i32> {
            let filled: Loan<'_, i32> = out.set(50);
            let val = *filled;
            let out: Out<'_, i32> = Loan::drop(filled);
            out.set(val * 2)
        });

        assert_eq!(*b, 100);
    }


    #[test]
    fn fibonacci() {
        let numbers: Box<[u64; 64]> = box_init(|out| {
            let mut slice = Slice::new(out);

            while !slice.is_full() {
                let v = match &*slice {
                    [.., a, b] => *a + *b,
                    _ => 1,
                };
                slice.push(v);
            }

            slice.assert_full()
        });

        assert_eq!(numbers.last(), Some(&10610209857723));
    }

    #[test]
    fn drop_own() {
        struct SetOnDrop<'r>(&'r mut bool);

        impl Drop for SetOnDrop<'_> {
            fn drop(&mut self) {
                *self.0 = true;
            }
        }

        let mut dropped = false;

        let slot = &mut stack_slot();
        let own = Own::new_in(slot, SetOnDrop(&mut dropped));
        drop(own);

        assert!(dropped);
    }
}
