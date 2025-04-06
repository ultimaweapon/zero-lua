pub use self::result::*;

use self::resume::Resume;
use super::Ret;
use crate::ffi::{LUA_OK, LUA_YIELD, engine_pop};
use crate::{Frame, FrameState, Str};
use std::cell::Cell;
use std::ffi::c_int;
use std::rc::Rc;
use std::task::Context;

mod result;
mod resume;

/// Struct to poll yieldable function.
pub struct AsyncCall<'a, P: Frame> {
    result: AsyncFrame<'a, P>,
    args: c_int,
    values: Rc<Cell<YieldValues>>,
    pending: Option<PendingFuture>,
}

impl<'a, P: Frame> AsyncCall<'a, P> {
    /// # Safety
    /// Top of stack must have `args` and below this must be a callable object.
    #[inline(always)]
    pub(super) unsafe fn new(parent: &'a mut P, args: c_int) -> Self {
        Self {
            result: AsyncFrame::new(parent),
            args,
            values: Rc::default(),
            pending: None,
        }
    }

    #[inline(always)]
    pub async fn resume<'b>(
        &'b mut self,
    ) -> Result<Async<'b, AsyncFrame<'a, P>>, Str<'b, AsyncFrame<'a, P>>>
    where
        'a: 'b,
    {
        let mut n = 0;
        let f = Resume::new(
            self.result.state(),
            &mut self.args,
            &self.values,
            &mut n,
            &mut self.pending,
        );

        match f.await {
            LUA_OK => unsafe { Ok(Async::Finish(Ret::new(&mut self.result, n))) },
            LUA_YIELD => unsafe { Ok(Async::Yield(Ret::new(&mut self.result, n))) },
            _ => unsafe { Err(Str::new(&mut self.result)) },
        }
    }
}

impl<'a, P: Frame> Drop for AsyncCall<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        if let Some(v) = self.pending.take() {
            unsafe { (v.drop)(v.future) };
        }

        // Pop stack.
        if self.args != 0 {
            unsafe { engine_pop(self.state().get(), self.args) };
        }

        unsafe { engine_pop(self.state().get(), 1) };
    }
}

impl<'a, P: Frame> FrameState for AsyncCall<'a, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&self) -> &Self::State {
        self.result.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        self.args += n;
    }
}

/// Results of polling yieldable function.
pub enum Async<'a, P: Frame> {
    Yield(Ret<'a, P>),
    Finish(Ret<'a, P>),
}

/// Context to poll yieldable function.
pub(crate) struct AsyncContext<'a, 'b> {
    pub cx: &'a mut Context<'b>,
    pub values: &'a Rc<Cell<YieldValues>>,
}

/// Encapsulates a number of value from/to Lua thread.
#[derive(Default, Clone, Copy)]
pub(crate) enum YieldValues {
    #[default]
    None,
    FromThread(c_int),
    ToThread(c_int),
}

/// RAII struct to drop pending a future.
struct PendingFuture {
    future: *mut (),
    drop: unsafe fn(f: *mut ()),
}
