use core::marker::PhantomData;
use core::ops::Deref;

use crate::mem::{Scope, ScopedMem};
use crate::{Init, Slot};

type Invariant<T> = PhantomData<core::cell::Cell<T>>;

pub struct ScopeGuard<'s>(Scope<'s>);

impl<'s> ScopeGuard<'s> {
    #[inline(always)]
    pub unsafe fn forge_scope() -> Scope<'s> {
        unsafe { Scope::forge() }
    }

    #[inline(always)]
    pub fn new(_: &'s Scope<'s>) -> Self {
        unsafe { ScopeGuard(Scope::forge()) }
    }
}

impl<'s> Drop for ScopeGuard<'s> {
    #[inline(always)]
    fn drop(&mut self) {}
}

#[macro_export]
macro_rules! let_scope {
    ($name:ident) => {
        let $name = unsafe { $crate::__::ScopeGuard::forge_scope() };
        let _guard = $crate::__::ScopeGuard::new(&$name);
    };
}

pub struct TypeMarker<T: ?Sized>(Invariant<T>);

impl<T: ?Sized> TypeMarker<T> {
    #[inline(always)]
    pub fn capture_type<V, S: Slot<V>>(slot: S) -> (S::Place, Self)
    where
        V: Deref<Target = T>,
    {
        (slot.into_place(), Self(PhantomData))
    }

    #[inline(always)]
    pub fn expect_init<'s>(&self, _: &'s Scope<'s>, init: Init<ScopedMem<'s, T>>) {
        core::mem::forget(init)
    }
}

#[macro_export]
macro_rules! emplace {
    ($slot:expr => $place:ident $block:block) => {{
        let $place = $slot;
        // SAFETY:
        // We make sure the place is never leaked or forgotten:
        //  - if `$block` diverges, it gets dropped;
        //  - otherwise, we it gets initialized;
        //  - this stays true if we're in a `async` block, as the block must be pinned
        //    to be executed, and `Pin` guarantees that the frame will be dropped.
        //
        // Not that the place type captured by TypeMarker isn't required for soundness;
        // it only gives better error messages.
        let (mut $place, _type) = $crate::__::TypeMarker::capture_type($place);
        {
            $crate::let_scope!(scope);
            let $place = scope.borrow(&mut $place);
            _type.expect_init(&scope, $block)
        }
        unsafe { $crate::Place::assume_init($place) }
    }};
}
