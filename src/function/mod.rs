pub use self::r#async::*;
pub use self::ret::*;

use crate::FrameState;
use crate::ffi::{engine_checkstack, engine_gettop, engine_pop, zl_pcall};
use crate::{AsyncState, Frame, MainState, Str};
use std::ffi::c_int;

mod r#async;
mod ret;

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
        let func = unsafe { engine_gettop(p.state().get()) };

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
    /// This will consume the callable object so it will not pushed to parent frame.
    #[inline(always)]
    pub fn call<R: FuncRet<'a, P>>(mut self) -> Result<R, Str<'a, P>> {
        // Ensure stack for results. We can't take out the parent here since engine_checkstack can
        // throw a C++ exception.
        if R::N > 0 {
            unsafe { engine_checkstack(self.parent.as_ref().unwrap().state().get(), R::N) };
        }

        // Call.
        let p = self.parent.take().unwrap();

        match unsafe { zl_pcall(p.state().get(), self.args, R::N, 0) } {
            true => Ok(unsafe { R::new(p, self.func - 1) }),
            false => Err(unsafe { Str::new(p) }),
        }
    }
}

impl<'a, P> Function<'a, P>
where
    P: Frame<State = AsyncState>,
{
    /// This will consume the callable object so it will not pushed to parent frame.
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
            unsafe { engine_pop(p.state().get(), self.args) };
        }

        unsafe { p.release_values(1) };
    }
}

impl<'a, P: Frame> FrameState for Function<'a, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&self) -> &Self::State {
        self.parent.as_ref().unwrap().state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        self.args += n;
    }
}
