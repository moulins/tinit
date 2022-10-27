use crate::mem::Lease;

#[doc(hidden)]
pub struct __LeaseGuard<'s>(Lease<'s>);

impl<'s> __LeaseGuard<'s> {
    #[inline(always)]
    pub unsafe fn forge_lease<'env>() -> Lease<'s> {
        unsafe { Lease::forge() }
    }

    #[inline(always)]
    pub fn new(_: &'s Lease<'s>) -> Self {
        unsafe { __LeaseGuard(Lease::forge()) }
    }
}

impl<'s> Drop for __LeaseGuard<'s> {
    #[inline(always)]
    fn drop(&mut self) {}
}

#[macro_export]
macro_rules! make_lease {
    ($name:ident) => {
        let $name = unsafe { $crate::__::__LeaseGuard::forge_lease() };
        let _guard = $crate::__::__LeaseGuard::new(&$name);
    };
}
