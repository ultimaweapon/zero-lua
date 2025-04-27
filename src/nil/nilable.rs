use crate::{Frame, FrameValue};
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
