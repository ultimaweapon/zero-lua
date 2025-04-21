pub(crate) use self::state::*;

use super::AsyncLua;
use crate::FrameState;
use crate::ffi::zl_pop;
use std::ffi::c_int;
use std::pin::Pin;
use std::rc::Rc;

mod state;

/// Encapsulates a `lua_State` created by `lua_newstate`.
pub struct Lua(MainState);

impl Lua {
    /// Create a new `lua_State` using `luaL_newstate`. Returns [`None`] if `luaL_newstate` return
    /// null.
    ///
    /// You may want to change Lua panic and warning function after this if your application is a
    /// GUI application.
    #[inline(always)]
    pub fn new() -> Option<Self> {
        MainState::new().map(Self)
    }

    pub fn into_async(self) -> Pin<Rc<AsyncLua>> {
        AsyncLua::new(self.0)
    }
}

impl FrameState for Lua {
    type State = MainState;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        &mut self.0
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.0.get(), n) };
    }
}
