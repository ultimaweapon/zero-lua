pub(crate) use self::state::*;

use super::MainState;
use crate::FrameState;
use crate::ffi::{ZL_REGISTRYINDEX, zl_newthread, zl_pop, zl_ref, zl_unref};
use std::ffi::c_int;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::rc::Rc;

mod state;

/// Provides method to create a Lua thread with ability to call into async function.
pub struct AsyncLua {
    state: MainState,
    _phantom: PhantomPinned,
}

impl AsyncLua {
    pub(super) fn new(main: MainState) -> Pin<Rc<Self>> {
        Rc::pin(Self {
            state: main,
            _phantom: PhantomPinned,
        })
    }

    pub fn spawn(self: &Pin<Rc<Self>>) -> AsyncThread {
        let state = unsafe { zl_newthread(self.state.get()) };
        let index = unsafe { zl_ref(self.state.get(), ZL_REGISTRYINDEX) };

        AsyncThread {
            main: self.clone(),
            state: unsafe { AsyncState::new(state) },
            index,
        }
    }
}

/// Encapsulates a Lua thread that can call into async function.
pub struct AsyncThread {
    main: Pin<Rc<AsyncLua>>,
    state: AsyncState,
    index: c_int,
}

impl Drop for AsyncThread {
    fn drop(&mut self) {
        unsafe { zl_unref(self.main.state.get(), ZL_REGISTRYINDEX, self.index) };
    }
}

impl FrameState for AsyncThread {
    type State = AsyncState;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        &mut self.state
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state.get(), n) };
    }
}
