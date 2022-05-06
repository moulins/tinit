use crate::token::{Token, Tokens};

impl<'s, T: Into<Token<'s>> + ?Sized> From<T> for Tokens<'s, 1> {
    #[inline]
    fn from(t: T) -> Self {
        Tokens([t.into()])
    }
}

impl<'s, const N: usize, T: Into<Token<'s>> + ?Sized> From<[T; N]> for Tokens<'s, N> {
    #[inline]
    fn from(array: [T; N]) -> Self {
        Tokens(array.map(Into::into))
    }
}

macro_rules! impl_tuple {
    ($n:literal; $($upper:ident $lower:ident;)*) => {
        impl<
            's,
            $($upper: Into<Token<'s>> + ?Sized,)*
        > From<( $($upper,)* )> for Tokens<'s, $n> {
            #[inline]
            fn from(( $($lower,)* ): ( $($upper,)* )) -> Self {
                Tokens([ $($lower.into(),)* ])
            }
        } 
    }
}

impl_tuple! { 0; }
impl_tuple! { 1; A a; }
impl_tuple! { 2; A a; B b; }
impl_tuple! { 3; A a; B b; C c; }
impl_tuple! { 4; A a; B b; C c; D d; }
impl_tuple! { 5; A a; B b; C c; D d; E e; }
impl_tuple! { 6; A a; B b; C c; D d; E e; F f; }
impl_tuple! { 7; A a; B b; C c; D d; E e; F f; G g; }
impl_tuple! { 8; A a; B b; C c; D d; E e; F f; G g; H h; }
impl_tuple! { 9; A a; B b; C c; D d; E e; F f; G g; H h; I i; }
impl_tuple! { 10; A a; B b; C c; D d; E e; F f; G g; H h; I i; J j; }
impl_tuple! { 11; A a; B b; C c; D d; E e; F f; G g; H h; I i; J j; K k; }
impl_tuple! { 12; A a; B b; C c; D d; E e; F f; G g; H h; I i; J j; K k; L l; }
