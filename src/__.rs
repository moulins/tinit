use core::marker::PhantomData;

use crate::mem::{Scope, ScopedMem};
use crate::{Init, IntoPlace};

type Invariant<T> = PhantomData<core::cell::Cell<T>>;

#[doc(hidden)]
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
macro_rules! make_scope {
    ($name:ident) => {
        let $name = unsafe { $crate::__::ScopeGuard::forge_scope() };
        let _guard = $crate::__::ScopeGuard::new(&$name);
    };
}

pub struct TypeMarker<T: ?Sized>(Invariant<T>);

impl<T: ?Sized> TypeMarker<T> {
    #[inline(always)]
    pub unsafe fn materialize<P: IntoPlace<Type = T>>(place: P) -> (P::Place, Self) {
        (unsafe { place.materialize() }, Self(PhantomData))
    }

    #[inline(always)]
    pub fn forget_out<'s>(&self, _: &'s Scope<'s>, out: Init<ScopedMem<'s, T>>) {
        core::mem::forget(out)
    }
}

#[macro_export]
macro_rules! emplace {
    ($place:expr => $out:ident $block:block) => {{
        let $out = $place;
        // SAFETY:
        // We make sure the place is never leaked or forgotten:
        //  - if `$block` diverges, it gets dropped;
        //  - otherwise, we it gets initialized;
        //  - this stays true if we're in a `async` block, as the block must be pinned
        //    to be executed, and `Pin` guarantees that the frame will be dropped.
        let (mut $out, _type) = unsafe { $crate::__::TypeMarker::materialize($out) };
        {
            $crate::make_scope!(scope);
            let $out = scope.borrow(&mut $out);
            let _init = $block;
            // Unfortunately, this suppresses warnings on all unreachable code in the rest
            // of the function, not just for this line.
            #[allow(unreachable_code)]
            {
                _type.forget_out(&scope, _init)
            }
        }
        unsafe { $crate::Place::finalize($out) }
    }};
}
