use crate::ffi::lua_State;

/// Encapsulates a `lua_State`.
///
/// This struct does not free the encapsulated value when dropped.
pub struct State(*mut lua_State);

impl State {
    #[inline(always)]
    pub fn new(v: *mut lua_State) -> Self {
        Self(v)
    }

    #[inline(always)]
    pub fn get(&self) -> *mut lua_State {
        self.0
    }
}
