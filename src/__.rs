use core::marker::PhantomData;

use crate::mem::Lease;
use crate::{Loan, Out, Place};

#[doc(hidden)]

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


pub struct EmplaceGuard<'s, T, P> {
    lease: &'s Lease<'s>,
    _invariant: PhantomData<core::cell::Cell<(P, T)>>,
}

impl<'s, T, P: Place<Type=T>> EmplaceGuard<'s, T, P> {
    #[inline(always)]
    pub fn for_place(lease: &'s Lease<'s>, _: &P) -> Self {
        Self { lease, _invariant: PhantomData }
    }

    #[inline(always)]
    pub fn borrow(&self, place: &'s mut P) -> Out<'s, T> {
        self.lease.borrow(place)
    }

    #[inline(always)]
    pub fn forget_loan(self, loan: Loan<'s, T>) {
        core::mem::forget(loan)
    }
}

#[macro_export]
macro_rules! emplace {
    ($place:expr => $out:ident $block:block) => {{
        let mut place = $place;
        {
            $crate::make_lease!(lease);
            let guard = $crate::__::EmplaceGuard::for_place(&lease, &place);
            let $out = guard.borrow(&mut place);
            guard.forget_loan($block);
        }
        unsafe { $crate::place::Place::assume_init(place) }
    }};
}
