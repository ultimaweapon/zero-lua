pub use self::extra::*;

use crate::ffi::{lua_State, zl_getextraspace};

mod extra;

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

    #[inline(always)]
    pub fn extra1(&self) -> &ExtraData {
        unsafe { &*zl_getextraspace(self.0).cast::<*const ExtraData>().read() }
    }

    #[inline(always)]
    pub fn extra2<T: Sized>(&self) -> *mut *mut T {
        unsafe { zl_getextraspace(self.0).add(1).cast() }
    }
}
