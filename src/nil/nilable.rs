use crate::Frame;
use crate::value::{FrameValue, IntoLua};
use std::num::NonZero;

/// Represents either nil or value on the top of stack.
pub enum Nilable<'a, T, P>
where
    T: FrameValue<'a, P>,
    P: Frame,
{
    Nil(&'a mut P),
    Value(T),
}

impl<'a, T, P> Drop for Nilable<'a, T, P>
where
    T: FrameValue<'a, P>,
    P: Frame,
{
    #[inline(always)]
    fn drop(&mut self) {
        if let Self::Nil(p) = self {
            unsafe { p.release_values(T::N.get().into()) };
        }
    }
}

unsafe impl<'a, T, P> FrameValue<'a, P> for Nilable<'a, T, P>
where
    T: FrameValue<'a, P>,
    P: Frame,
{
    const N: NonZero<u8> = T::N;
}

unsafe impl<T: IntoLua> IntoLua for Option<T> {
    type Value<'a, P: Frame + 'a> = Nilable<'a, T::Value<'a, P>, P>;

    #[inline(always)]
    fn into_lua<P: Frame>(self, p: &mut P) -> Self::Value<'_, P> {
        match self {
            Some(v) => Nilable::Value(v.into_lua(p)),
            None => {
                for _ in 0..<T::Value<'_, P> as FrameValue<P>>::N.get() {
                    p.push_nil();
                }

                Nilable::Nil(p)
            }
        }
    }
}
