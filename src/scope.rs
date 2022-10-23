use core::cell::Cell;
use core::marker::PhantomData;
use core::ptr::NonNull;

type Invariant<T> = PhantomData<Cell<T>>;

/// A scope with an invariant lifetime.
///
/// Useful for constructing [`Out`][crate::Out] and [`Slot`][crate::Slot] instances.
pub struct Scope<'s>(Invariant<&'s ()>);

/// Enters a new scope with a fresh lifetime.
pub fn enter<F, R>(f: F) -> R
where
    F: for<'s> FnOnce(Scope<'s>) -> R,
{
    f(Scope(PhantomData))
}

impl<'s> Scope<'s> {
    pub(crate) fn with_ptr<T: ?Sized>(&self, ptr: NonNull<T>) -> ScopedPtr<'s, T> {
        ScopedPtr {
            ptr,
            _marker: PhantomData,
        }
    }
}

// TODO: should this type be exposed in the public API?
pub(crate) struct ScopedPtr<'s, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: Invariant<&'s T>,
}

impl<'s, T: ?Sized> std::ops::Deref for ScopedPtr<'s, T> {
    type Target = NonNull<T>;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

impl<'s, T: ?Sized> std::ops::DerefMut for ScopedPtr<'s, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ptr
    }
}

impl<'s, T: ?Sized> Clone for ScopedPtr<'s, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'s, T: ?Sized> Copy for ScopedPtr<'s, T> {}
