use crate::ffi::{zl_close, zl_getextraspace, zl_newstate};
use crate::state::{ExtraData, State};
use std::ops::{Deref, DerefMut};

/// Encapsulates [`State`] created from `lua_newstate`.
pub struct MainState(State);

impl MainState {
    pub(super) fn new(panic: Box<dyn Fn(Option<&str>)>) -> Option<Self> {
        // Create lua_State.
        let state = zl_newstate();
        let state = if state.is_null() {
            return None;
        } else {
            Self(State::new(state))
        };

        // Set extra data.
        let space = unsafe { zl_getextraspace(state.get()).cast::<*mut ExtraData>() };
        let extra = Box::new(ExtraData { panic });

        unsafe { space.write(Box::into_raw(extra)) };

        Some(state)
    }
}

impl Drop for MainState {
    fn drop(&mut self) {
        // Free extra data.
        let extra = unsafe { zl_getextraspace(self.get()).cast::<*mut ExtraData>() };
        let extra = unsafe { extra.read() };

        if !extra.is_null() {
            drop(unsafe { Box::from_raw(extra) });
        }

        // Free lua_State.
        unsafe { zl_close(self.0.get()) };
    }
}

impl Deref for MainState {
    type Target = State;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MainState {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
