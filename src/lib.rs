#![no_std]
#![warn(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::boxed::Box;
use core::mem::MaybeUninit;

/// Macro definitions and utilities.
#[doc(hidden)]
#[macro_use]
pub mod __;

// Private modules
mod init;
mod polyfill;

// Public modules
pub mod mem;
pub mod place;
pub mod slice;
pub mod uninit;

// Reexports
#[doc(no_inline)]
pub use mem::{Mem, ScopedMem};
#[doc(no_inline)]
pub use place::{Slot, Place};

pub use init::Init;

pub type Own<'s, T> = Init<Mem<'s, T>>;
pub type ScopedRef<'s, T> = Init<mem::ScopedMem<'s, T>>;

// TODO: constify everything that can be constified.

#[inline(always)]
pub fn emplace_box<T>() -> impl Slot<Box<T>> {
    use uninit::{UninitMut, UninitRef};

    struct BoxPlace<T>(Box<MaybeUninit<T>>);

    unsafe impl<T> Place for BoxPlace<T> {
        type Target = T;
        type Init = Box<T>;

        #[inline(always)]
        fn deref_place(&self) -> UninitRef<'_, Self::Target> {
            (&*self.0).into()
        }

        #[inline(always)]
        fn deref_place_mut(&mut self) -> UninitMut<'_, Self::Target> {
            (&mut *self.0).into()
        }

        #[inline(always)]
        unsafe fn assume_init(self) -> Self::Init {
            unsafe { polyfill::box_assume_init(self.0) }
        }
    }

    BoxPlace(polyfill::box_new_uninit())
}

#[inline(always)]
pub fn slot<T>() -> MaybeUninit<T> {
    MaybeUninit::uninit()
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use slice::Slice;

    #[test]
    fn it_works() {
        let b: Box<i32> = emplace!(emplace_box() => out {
            let filled: Init<ScopedMem<'_, i32>> = out.set(50);
            let val = *filled;
            let out: ScopedMem<'_, i32> = Init::drop(filled);
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

        let slot = &mut slot();
        let own = Own::new_in(slot, SetOnDrop(&mut dropped));
        drop(own);

        assert!(dropped);
    }
}
