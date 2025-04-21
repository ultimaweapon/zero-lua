pub use self::r#async::*;
pub use self::result::*;

use crate::FrameState;
use crate::ffi::{LUA_MULTRET, zl_gettop, zl_pcall, zl_pop};
use crate::{AsyncState, Frame, MainState, Str};
use std::ffi::c_int;

mod r#async;
mod result;

/// Encapsulates a callable object on the top of Lua stack.
pub struct Function<'a, P: Frame> {
    parent: Option<&'a mut P>,
    func: c_int,
    args: c_int,
}

impl<'a, P: Frame> Function<'a, P> {
    /// # Safety
    /// Top of the stack must be a callable object.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        // TODO: Find a way to eliminate this for FixedRet.
        let func = unsafe { zl_gettop(p.state().get()) };

        Self {
            parent: Some(p),
            func,
            args: 0,
        }
    }
}

impl<'a, P> Function<'a, P>
where
    P: Frame<State = MainState>,
{
    /// This will consume the callable object so it will not pushed to the parent frame when
    /// dropped.
    #[inline(always)]
    pub fn call(mut self) -> Result<Ret<'a, P>, Str<'a, P>> {
        // Call.
        let p = self.parent.take().unwrap();

        if !unsafe { zl_pcall(p.state().get(), self.args, LUA_MULTRET, 0) } {
            return Err(unsafe { Str::new(p) });
        }

        // Get results.
        let l = unsafe { zl_gettop(p.state().get()) - (self.func - 1) };

        Ok(unsafe { Ret::new(p, l) })
    }
}

impl<'a, P> Function<'a, P>
where
    P: Frame<State = AsyncState>,
{
    /// This will consume the callable object so it will not pushed to the parent frame when
    /// dropped.
    #[inline(always)]
    pub fn into_async(mut self) -> AsyncCall<'a, P> {
        unsafe { AsyncCall::new(self.parent.take().unwrap(), self.args) }
    }
}

impl<'a, P: Frame> Drop for Function<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        let p = match self.parent.take() {
            Some(v) => v,
            None => return,
        };

        if self.args != 0 {
            unsafe { zl_pop(p.state().get(), self.args) };
        }

        unsafe { p.release_values(1) };
    }
}

impl<'a, P: Frame> FrameState for Function<'a, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.parent.as_mut().unwrap().state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        self.args += n;
    }
}
