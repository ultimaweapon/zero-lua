use crate::ffi::{zl_close, zl_newstate};
use crate::state::State;
use std::ops::{Deref, DerefMut};

/// Encapsulates [`State`] created from `lua_newstate`.
pub struct MainState(State);

impl MainState {
    #[inline(always)]
    pub(super) fn new() -> Option<Self> {
        let state = zl_newstate();

        if state.is_null() {
            None
        } else {
            Some(Self(State::new(state)))
        }
    }
}

impl Drop for MainState {
    #[inline(always)]
    fn drop(&mut self) {
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
