use super::{AsyncContext, PendingFuture, YieldValues};
use crate::ffi::{LUA_YIELD, engine_pop, engine_touserdata, zl_getextraspace, zl_resume};
use crate::state::State;
use std::cell::Cell;
use std::ffi::c_int;
use std::marker::PhantomData;
use std::mem::transmute;
use std::pin::Pin;
use std::ptr::null_mut;
use std::rc::Rc;
use std::task::{Context, Poll};

/// Implementation of [`Future`] to poll yieldable function.
pub struct Resume<'a> {
    state: &'a State,
    args: &'a mut c_int,
    values: &'a Rc<Cell<YieldValues>>,
    results: &'a mut c_int,
    pending: &'a mut Option<PendingFuture>,
}

impl<'a> Resume<'a> {
    #[inline(always)]
    pub(super) fn new(
        state: &'a State,
        args: &'a mut c_int,
        values: &'a Rc<Cell<YieldValues>>,
        results: &'a mut c_int,
        pending: &'a mut Option<PendingFuture>,
    ) -> Self {
        Self {
            state,
            args,
            values,
            results,
            pending,
        }
    }
}

impl<'a> Future for Resume<'a> {
    type Output = c_int;

    #[inline(never)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Setup context.
        let mut cx = AsyncContext {
            cx,
            values: self.values,
        };

        // Check if first call.
        let mut args = std::mem::take(self.args);

        if self.pending.take().is_some() {
            match cx.values.get() {
                YieldValues::None => {
                    if args > 0 {
                        // The pending future does not prepare for this so we need to remove it
                        // here.
                        unsafe { engine_pop(self.state.get(), args) };
                        args = 0;
                    }
                }
                YieldValues::FromThread(_) => cx.values.set(YieldValues::ToThread(args)),
                YieldValues::ToThread(_) => unreachable!(),
            }
        }

        // We forbid async call within LocalState so "from" always null here.
        let l = unsafe { ContextLock::new(self.state, &mut cx) };
        let r = unsafe { zl_resume(self.state.get(), null_mut(), args, self.results) };

        drop(l);

        if r != LUA_YIELD {
            return Poll::Ready(r);
        }

        // Check if yield from our invokder.
        let cx = &mut cx as *mut AsyncContext as *mut u8;

        if *self.results != 3 || unsafe { engine_touserdata(self.state.get(), -1) != cx } {
            return Poll::Ready(LUA_YIELD);
        }

        // Keep pending future.
        let future = unsafe { engine_touserdata(self.state.get(), -2).cast() };
        let drop = unsafe { engine_touserdata(self.state.get(), -3) };

        *self.pending = Some(PendingFuture {
            future,
            drop: unsafe { transmute(drop) },
        });

        unsafe { engine_pop(self.state.get(), 3) };

        // Check how we yield.
        match self.values.get() {
            YieldValues::None => Poll::Pending,
            YieldValues::FromThread(v) => {
                *self.results = v;
                self.values.set(YieldValues::FromThread(0)); // Prevent double free on future side.
                Poll::Ready(LUA_YIELD)
            }
            YieldValues::ToThread(_) => unreachable!(),
        }
    }
}

/// RAII struct to clear extra space from `lua_State`.
struct ContextLock<'a, 'b, 'c> {
    ptr: *mut *mut AsyncContext<'b, 'c>,
    phantom: PhantomData<&'a State>,
}

impl<'a, 'b, 'c> ContextLock<'a, 'b, 'c> {
    /// # Safety
    /// Extra space must be a pointer size and it must not contains any data.
    #[inline(always)]
    unsafe fn new(state: &'a State, cx: &'a mut AsyncContext<'b, 'c>) -> Self {
        let ptr = unsafe { zl_getextraspace(state.get()).cast::<*mut AsyncContext>() };

        unsafe { ptr.write(cx) };

        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a, 'b, 'c> Drop for ContextLock<'a, 'b, 'c> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.ptr.write(null_mut()) };
    }
}
