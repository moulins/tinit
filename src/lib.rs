#![no_std]
#![warn(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::boxed::Box;
use core::mem::MaybeUninit;

/// Macro definitions and utilities.
#[doc(hidden)]
#[macro_use]
pub mod __;

pub mod init;
pub mod mem;
pub mod place;
mod polyfill;
pub mod slice;

pub use init::Init;
pub use place::{IntoPlace, Place};

pub type Slot<'s, T> = &'s mut MaybeUninit<T>;
pub type Uninit<'s, T> = mem::OwnedMem<'s, T>;
pub type Own<'s, T> = Init<mem::OwnedMem<'s, T>>;
pub type Out<'s, T> = mem::LeasedMem<'s, T>;
pub type Loan<'s, T> = Init<mem::LeasedMem<'s, T>>;

// TODO: constify everything that can be constified.

#[inline(always)]
pub fn emplace_box<T>() -> impl IntoPlace<Type = T, Init = Box<T>> {
    struct BoxPlace<T>(Box<MaybeUninit<T>>);

    unsafe impl<T> Place for BoxPlace<T> {
        type Type = T;

        type Init = Box<T>;

        #[inline(always)]
        fn as_ptr(&self) -> *const Self::Type {
            self.0.as_ptr()
        }

        #[inline(always)]
        fn as_mut_ptr(&mut self) -> *mut Self::Type {
            self.0.as_mut_ptr()
        }

        #[inline(always)]
        unsafe fn assume_init(self) -> Self::Init {
            unsafe { polyfill::box_assume_init(self.0) }
        }
    }

    place::PlaceFn(|| BoxPlace(polyfill::box_new_uninit()))
}

#[inline(always)]
pub fn stack_slot<T>() -> MaybeUninit<T> {
    MaybeUninit::uninit()
}

#[cfg(test)]
mod tests {
    use super::*;
    use slice::Slice;

    #[test]
    fn it_works() {
        let b: Box<i32> = emplace!(emplace_box() => out {
            let filled: Loan<'_, i32> = out.set(50);
            let val = *filled;
            let out: Out<'_, i32> = Loan::drop(filled);
            out.set(val * 2)
        });

        assert_eq!(*b, 100);
    }

    #[test]
    fn fibonacci() {
        let numbers: Box<[u64; 64]> = emplace!(emplace_box() => out {
            let mut slice = Slice::new(out);

            while !slice.is_full() {
                let v = match &*slice {
                    [a, b, ..] => *a + *b,
                    _ => 1,
                };
                slice.emplace_at(0).set(v);
            }

            slice.assert_full()
        });

        assert_eq!(numbers.first(), Some(&10610209857723));
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
