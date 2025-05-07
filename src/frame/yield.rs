use crate::ffi::{lua_State, zl_pop};
use crate::state::RawState;
use crate::{Context, LocalState, Ret, YieldValues, Yieldable};
use std::cell::Cell;
use std::ffi::c_int;
use std::future::poll_fn;
use std::ops::Deref;
use std::task::Poll;

/// Provides method to yield from Lua thread.
pub struct Yield<'a, 'b> {
    parent: Option<&'a mut Context<'b, Yieldable>>,
    values: c_int,
}

impl<'a, 'b> Yield<'a, 'b> {
    #[inline(always)]
    pub(crate) fn new(parent: &'a mut Context<'b, Yieldable>) -> Self {
        Self {
            parent: Some(parent),
            values: 0,
        }
    }

    pub async fn yield_now(mut self) -> Ret<'a, Context<'b, Yieldable>> {
        // Set values to yield.
        let parent = self.parent.take().unwrap();
        let values = ValuesGuard(parent.state());

        values.set(YieldValues::FromThread(std::mem::take(&mut self.values)));

        // Setup future for return values.
        let f = poll_fn(move |_| match values.get() {
            YieldValues::None => unreachable!(),
            YieldValues::FromThread(_) => Poll::Pending,
            YieldValues::ToThread(v) => {
                values.set(YieldValues::None);
                Poll::Ready(v)
            }
        });

        // Wait for return values.
        let n = f.await;

        unsafe { Ret::new(parent, n) }
    }
}

impl Drop for Yield<'_, '_> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.values != 0 {
            unsafe { zl_pop(self.state(), self.values) };
        }
    }
}

impl RawState for Yield<'_, '_> {
    #[inline(always)]
    fn state(&mut self) -> *mut lua_State {
        self.parent.as_mut().unwrap().state().get()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        self.values += n;
    }
}

/// RAII struct to remove yield values.
struct ValuesGuard<'a>(&'a mut Yieldable);

impl Drop for ValuesGuard<'_> {
    #[inline(always)]
    fn drop(&mut self) {
        let n = match Yieldable::values(self.0).take() {
            YieldValues::None => return,
            YieldValues::FromThread(v) => v,
            YieldValues::ToThread(v) => v,
        };

        if n != 0 {
            unsafe { zl_pop(self.0.get(), n) };
        }
    }
}

impl Deref for ValuesGuard<'_> {
    type Target = Cell<YieldValues>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        Yieldable::values(self.0)
    }
}
