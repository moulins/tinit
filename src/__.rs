use crate::mem::Lease;
use crate::Loan;

#[doc(hidden)]
pub struct LeaseGuard<'s>(Lease<'s>);

impl<'s> LeaseGuard<'s> {
    #[inline(always)]
    pub unsafe fn forge_lease<'env>() -> Lease<'s> {
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

#[inline(always)]
pub fn forget_leased_loan<'s, T>(_lease: &Lease<'s>, loan: Loan<'s, T>) {
    core::mem::forget(loan)
}

#[macro_export]
macro_rules! emplace {
    ($place:expr => $out:ident $block:block) => {{
        $crate::make_lease!(lease);
        let mut place = $place;
        let ptr = $crate::place::Place::as_mut_ptr(&mut place);
        let $out: $crate::Out<'_, _> = unsafe { lease.borrow_ptr(ptr) };
        $crate::__::forget_leased_loan(&lease, $block);
        unsafe { $crate::place::Place::assume_init(place) }
    }};
}
