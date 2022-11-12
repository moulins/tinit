use core::marker::PhantomData;

use crate::mem::Lease;
use crate::{Loan, Place};

type Invariant<T> = PhantomData<core::cell::Cell<T>>;

#[doc(hidden)]
pub struct LeaseGuard<'s>(Lease<'s>);

impl<'s> LeaseGuard<'s> {
    #[inline(always)]
    pub unsafe fn forge_lease() -> Lease<'s> {
        unsafe { Lease::forge() }
    }

    #[inline(always)]
    pub fn new(_: &'s Lease<'s>) -> Self {
        unsafe { LeaseGuard(Lease::forge()) }
    }
}

impl<'s> Drop for LeaseGuard<'s> {
    #[inline(always)]
    fn drop(&mut self) {}
}

#[macro_export]
macro_rules! make_lease {
    ($name:ident) => {
        let $name = unsafe { $crate::__::LeaseGuard::forge_lease() };
        let _guard = $crate::__::LeaseGuard::new(&$name);
    };
}

pub struct TypeMarker<T: ?Sized>(Invariant<T>);

impl<T: ?Sized> TypeMarker<T> {
    #[inline(always)]
    pub fn assert_place<P: Place<Type=T>>(place: P) -> (P, Self) {
        (place, Self(PhantomData))
    }

    #[inline(always)]
    pub fn forget_loan<'s>(&self, _: &'s Lease<'s>, loan: Loan<'s, T>) {
        core::mem::forget(loan)
    }
}

#[macro_export]
macro_rules! emplace {
    ($place:expr => $out:ident $block:block) => {{
        let (mut $out, _place_type) = $crate::__::TypeMarker::assert_place($place);
        {
            $crate::make_lease!(lease);
            let $out = lease.borrow(&mut $out);
            let _loan = $block;
            // Unfortunately, this suppresses warnings on all unreachable code in the rest
            // of the function, not just for this line.
            #[allow(unreachable_code)]
            { _place_type.forget_loan(&lease, _loan) }
        }
        unsafe { $crate::place::Place::assume_init($out) }
    }};
}
