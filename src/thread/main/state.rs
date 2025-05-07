use crate::PanicHandler;
use crate::ffi::{lua_State, zl_close, zl_getextraspace, zl_newstate};
use crate::state::ExtraData;

/// Encapsulates [`State`] created from `lua_newstate`.
pub struct MainState(*mut lua_State);

impl MainState {
    pub(super) fn new(panic: Box<PanicHandler>) -> Option<Self> {
        // Create lua_State.
        let state = zl_newstate();
        let state = if state.is_null() {
            return None;
        } else {
            Self(state)
        };

        // Set extra data.
        let space = unsafe { zl_getextraspace(state.0).cast::<*mut ExtraData>() };
        let extra = Box::new(ExtraData { panic });

        unsafe { space.write(Box::into_raw(extra)) };

        Some(state)
    }

    pub fn get(&self) -> *mut lua_State {
        self.0
    }
}

impl Drop for MainState {
    fn drop(&mut self) {
        // Free extra data.
        let extra = unsafe { zl_getextraspace(self.0).cast::<*mut ExtraData>() };
        let extra = unsafe { extra.read() };

        if !extra.is_null() {
            drop(unsafe { Box::from_raw(extra) });
        }

        // Free lua_State.
        unsafe { zl_close(self.0) };
    }
}
