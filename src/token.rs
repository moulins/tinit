
// Nested module to enforce safety barrier.
mod types {
    use core::marker::PhantomData;

    type Invariant<'a> = PhantomData<*mut &'a ()>;

    /// A scope with an invariant lifetime.
    ///
    /// Useful for constructing [`Out`][crate::Out] and [`Slot`][crate::Slot] instances.
    #[derive(Copy, Clone, Debug)]
    pub struct Scope<'s>(Invariant<'s>);

    /// A token proving that an [`Out`][crate::Out] was properly initialized.
    #[derive(Debug)]
    pub struct Token<'s>(Invariant<'s>);

    #[inline]
    pub(super) unsafe fn mk_scope<'s>() -> Scope<'s> {
        Scope(PhantomData)
    }

    #[inline]
    pub(super) unsafe fn mk_token<'s>() -> Token<'s> {
        Token(PhantomData)
    }
}

pub use types::{Scope, Token};

/// Enters a new scope with a fresh lifetime.
pub fn scope<F, R>(f: F) -> R
where
    F: for<'s> FnOnce(Scope<'s>) -> R,
{
    f(unsafe { types::mk_scope() })
}

impl<'s> Token<'s> {
    #[inline]
    pub fn new(scope: Scope<'s>) -> Self {
        let _ = scope;
        unsafe { types::mk_token() }
    }
}

pub struct Tokens<'s, const N: usize>(pub [Token<'s>; N]);

/*
    struct S<T> {
        e: T
    }

    emplace_fields! {
        let S::<T> { e: f } in out => {
            f.fill(42).into()
        }
    }
*/
