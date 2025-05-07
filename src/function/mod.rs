pub use self::r#async::*;
pub use self::result::*;

use crate::ffi::{LUA_MULTRET, zl_gettop, zl_pcall, zl_pop};
use crate::state::RawState;
use crate::{AsyncState, Frame, MainState, Str, Unknown};
use std::ffi::c_int;

mod r#async;
mod result;

/// Encapsulates a callable object on the top of Lua stack.
pub struct Function<'p, P: Frame> {
    parent: Option<&'p mut P>,
    func: c_int,
    args: c_int,
}

impl<'p, P: Frame> Function<'p, P> {
    /// # Safety
    /// Top of the stack must be a callable object.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'p mut P) -> Self {
        // TODO: Find a way to eliminate this for FixedRet.
        let func = unsafe { zl_gettop(p.state().get()) };

        Self {
            parent: Some(p),
            func,
            args: 0,
        }
    }

    #[inline(always)]
    pub fn into_unknown(mut self) -> Unknown<'p, P> {
        let p = self.parent.take().unwrap();

        if self.args != 0 {
            unsafe { zl_pop(p.state().get(), self.args) };
        }

        unsafe { Unknown::new(p) }
    }
}

impl<'p, P> Function<'p, P>
where
    P: Frame<State = MainState>,
{
    /// This will consume the callable object so it will not pushed to the parent frame when
    /// dropped.
    #[inline(always)]
    pub fn call(mut self) -> Result<Ret<'p, P>, Str<'p, P>> {
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

impl<'p, P> Function<'p, P>
where
    P: Frame<State = AsyncState>,
{
    /// This will consume the callable object so it will not pushed to the parent frame when
    /// dropped.
    #[inline(always)]
    pub fn into_async(mut self) -> AsyncCall<'p, P> {
        unsafe { AsyncCall::new(self.parent.take().unwrap(), self.args) }
    }
}

impl<P: Frame> Drop for Function<'_, P> {
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

impl<P: Frame> RawState for Function<'_, P> {
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

impl<'p, P: Frame> From<Function<'p, P>> for Unknown<'p, P> {
    #[inline(always)]
    fn from(value: Function<'p, P>) -> Self {
        value.into_unknown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChunkType, Lua};

    #[test]
    fn async_resume_complete_immediately() {
        let mut lua = Lua::new(None).unwrap().into_async().spawn();

        pollster::block_on(async {
            let mut f = lua
                .load(None, ChunkType::Text, b"return 5")
                .unwrap()
                .into_async();
            let mut r = match f.resume().await.unwrap() {
                Async::Yield(_) => panic!("unexpected yield"),
                Async::Finish(v) => v,
            };

            assert_eq!(r.to_int(1).unwrap(), 5);
        });
    }
}
