use super::{Frame, FrameState};
use crate::ffi::engine_pop;
use crate::{DynamicRet, YieldValues, Yieldable};
use std::ffi::c_int;
use std::future::poll_fn;
use std::task::Poll;

/// Provides method to yield from Lua thread.
pub struct Yield<'a, P: Frame> {
    parent: Option<&'a mut P>,
    values: c_int,
}

impl<'a, P> Yield<'a, P>
where
    P: Frame<State = Yieldable>,
{
    #[inline(always)]
    pub(super) fn new(parent: &'a mut P) -> Self {
        Self {
            parent: Some(parent),
            values: 0,
        }
    }

    pub async fn yield_now(mut self) -> DynamicRet<'a, P> {
        // Set values to yield.
        let parent = self.parent.take().unwrap();
        let values = Yieldable::values(parent.state());

        values.set(YieldValues::FromThread(std::mem::take(&mut self.values)));

        // Setup future for return values.
        let guard = ValuesGuard(parent.state());
        let f = poll_fn(|_| match values.get() {
            YieldValues::None => unreachable!(),
            YieldValues::FromThread(_) => Poll::Pending,
            YieldValues::ToThread(v) => {
                values.set(YieldValues::None);
                Poll::Ready(v)
            }
        });

        // Wait for return values.
        let n = f.await;

        drop(guard);

        unsafe { DynamicRet::with_values(parent, n) }
    }
}

impl<'a, P: Frame> Drop for Yield<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.values != 0 {
            unsafe { engine_pop(self.state().get(), self.values) };
        }
    }
}

impl<'a, P: Frame> FrameState for Yield<'a, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&self) -> &Self::State {
        self.parent.as_ref().unwrap().state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        self.values += n;
    }
}

/// RAII struct to remove yield values.
struct ValuesGuard<'a>(&'a Yieldable);

impl<'a> Drop for ValuesGuard<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        let n = match Yieldable::values(self.0).take() {
            YieldValues::None => return,
            YieldValues::FromThread(v) => v,
            YieldValues::ToThread(v) => v,
        };

        if n != 0 {
            unsafe { engine_pop(self.0.get(), n) };
        }
    }
}
