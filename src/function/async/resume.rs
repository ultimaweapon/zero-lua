use super::{AsyncContext, PendingFuture, YieldValues};
use crate::ffi::{LUA_YIELD, zl_pop, zl_resume, zl_touserdata};
use crate::state::State;
use std::cell::Cell;
use std::ffi::c_int;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::null_mut;
use std::rc::Rc;
use std::task::{Context, Poll};

/// Implementation of [`Future`] to poll yieldable function.
pub struct Resume<'a> {
    state: &'a mut State,
    args: &'a mut c_int,
    values: &'a Rc<Cell<YieldValues>>,
    results: &'a mut c_int,
    pending: &'a mut Option<PendingFuture>,
}

impl<'a> Resume<'a> {
    #[inline(always)]
    pub(super) fn new(
        state: &'a mut State,
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

impl Future for Resume<'_> {
    type Output = c_int;

    #[inline(never)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Setup context.
        let this = self.get_mut();
        let mut cx = AsyncContext {
            cx,
            values: this.values,
        };

        // Check if first call.
        let mut args = std::mem::take(this.args);

        if this.pending.take().is_some() {
            match cx.values.get() {
                YieldValues::None => {
                    if args > 0 {
                        // The pending future does not prepare for this so we need to remove it
                        // here.
                        unsafe { zl_pop(this.state.get(), args) };
                        args = 0;
                    }
                }
                YieldValues::FromThread(_) => cx.values.set(YieldValues::ToThread(args)),
                YieldValues::ToThread(_) => unreachable!(),
            }
        }

        // We forbid async call within LocalState so "from" always null here.
        let l = unsafe { ContextLock::new(this.state, &mut cx) };
        let r = unsafe { zl_resume(l.get(), null_mut(), args, this.results) };

        drop(l);

        if r != LUA_YIELD {
            return Poll::Ready(r);
        }

        // Check if yield from our invokder.
        let cx = &mut cx as *mut AsyncContext as *mut u8;

        if *this.results != 3 || unsafe { zl_touserdata(this.state.get(), -1) != cx } {
            return Poll::Ready(LUA_YIELD);
        }

        // Keep pending future.
        let future = unsafe { zl_touserdata(this.state.get(), -2).cast() };
        let drop = unsafe { zl_touserdata(this.state.get(), -3) };

        *this.pending = Some(PendingFuture {
            future,
            drop: unsafe { transmute::<*mut u8, unsafe fn(*mut ())>(drop) },
        });

        unsafe { zl_pop(this.state.get(), 3) };

        // Check how we yield.
        match this.values.get() {
            YieldValues::None => Poll::Pending,
            YieldValues::FromThread(v) => {
                *this.results = v;
                this.values.set(YieldValues::FromThread(0)); // Prevent double free on future side.
                Poll::Ready(LUA_YIELD)
            }
            YieldValues::ToThread(_) => unreachable!(),
        }
    }
}

/// RAII struct to clear extra space from `lua_State`.
struct ContextLock<'a, 'b, 'c> {
    st: &'a mut State,
    cx: *mut *mut AsyncContext<'b, 'c>,
}

impl<'a, 'b, 'c> ContextLock<'a, 'b, 'c> {
    /// # Safety
    /// Extra space must be a pointer size and it must not contains any data.
    #[inline(always)]
    unsafe fn new(st: &'a mut State, cx: &'a mut AsyncContext<'b, 'c>) -> Self {
        let ptr = st.extra2::<AsyncContext>();

        unsafe { ptr.write(cx) };

        Self { st, cx: ptr }
    }
}

impl Drop for ContextLock<'_, '_, '_> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.cx.write(null_mut()) };
    }
}

impl Deref for ContextLock<'_, '_, '_> {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        self.st
    }
}

impl DerefMut for ContextLock<'_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.st
    }
}
