pub use self::extra::*;

use crate::ffi::{lua_State, zl_getextraspace};
use std::ffi::c_int;

mod extra;

/// Provides method to get `lua_State`.
pub trait RawState: Sized {
    fn state(&mut self) -> *mut lua_State;

    /// # Safety
    /// `n` must be greater than zero and `n` values on the top of stack must be owned by the
    /// caller.
    unsafe fn release_values(&mut self, n: c_int);

    #[inline(always)]
    fn extra1(&mut self) -> &ExtraData {
        let ptr = unsafe { zl_getextraspace(self.state()).cast::<*const ExtraData>() };

        unsafe { &*ptr.read() }
    }

    #[inline(always)]
    fn extra2<T: Sized>(&mut self) -> *mut *mut T {
        unsafe { zl_getextraspace(self.state()).add(1).cast() }
    }
}
