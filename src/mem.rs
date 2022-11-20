pub use base::Mem;
pub use heap::BoxMem;
pub use scoped::{Scope, ScopedMem};

macro_rules! impl_place_deref {
    (use $field:tt) => {
        #[inline(always)]
        fn deref_place(&self) -> $crate::uninit::UninitRef<'_, Self::Target> {
            self.$field.deref_place()
        }

        #[inline(always)]
        fn deref_place_mut(&mut self) -> $crate::uninit::UninitMut<'_, Self::Target> {
            self.$field.deref_place_mut()
        }
    };
}

mod base;
mod heap;
mod scoped;

