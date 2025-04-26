use crate::Frame;
use crate::ffi::zl_pushboolean;
use crate::value::{FrameValue, IntoLua};
use std::num::NonZero;

/// Represents a boolean on the top of stack.
pub struct Bool<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Bool<'a, P> {
    /// # Safety
    /// Top of the stack must be a boolean.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}

impl<'a, P: Frame> Drop for Bool<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(Self::N.get().into()) };
    }
}

unsafe impl<'a, P: Frame> FrameValue<'a, P> for Bool<'a, P> {
    const N: NonZero<u8> = NonZero::new(1).unwrap();
}

unsafe impl IntoLua for bool {
    type Value<'a, P: Frame + 'a> = Bool<'a, P>;

    #[inline(always)]
    fn into_lua<P: Frame>(self, p: &mut P) -> Self::Value<'_, P> {
        unsafe { zl_pushboolean(p.state().get(), self) };
        Bool(p)
    }
}
