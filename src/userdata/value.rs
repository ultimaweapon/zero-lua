use super::{UserData, is_boxed, push_metatable};
use crate::ffi::{zl_newuserdatauv, zl_pop, zl_setmetatable};
use crate::value::{FrameValue, IntoLua};
use crate::{Frame, FrameState};
use std::ffi::c_int;
use std::num::NonZero;

/// Represents a user data in a frame.
pub struct UserValue<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> UserValue<'a, P> {
    /// # Safety
    /// Top of the stack must be a strongly typed user data.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}

impl<'a, P: Frame> Drop for UserValue<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(Self::N.get().into()) };
    }
}

unsafe impl<'a, P: Frame> FrameValue<'a, P> for UserValue<'a, P> {
    const N: NonZero<u8> = NonZero::new(1).unwrap();
}

impl<'a, P: Frame> FrameState for UserValue<'a, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.0.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state().get(), n) };
    }
}

unsafe impl<T: UserData> IntoLua for T {
    type Value<'a, P: Frame + 'a> = UserValue<'a, P>;

    fn into_lua<P: Frame>(self, p: &mut P) -> Self::Value<'_, P> {
        if is_boxed::<T>() {
            let ptr = unsafe { zl_newuserdatauv(p.state().get(), size_of::<Box<T>>(), 0) };

            unsafe { ptr.cast::<Box<T>>().write(self.into()) };
        } else {
            let ptr = unsafe { zl_newuserdatauv(p.state().get(), size_of::<T>(), 0) };

            unsafe { ptr.cast::<T>().write(self) };
        }

        unsafe { push_metatable::<T>(p.state().get()) };
        unsafe { zl_setmetatable(p.state().get(), -2) };

        UserValue(p)
    }
}
