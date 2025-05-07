use crate::Frame;
use crate::ffi::{lua_State, zl_pop, zl_pushvalue, zl_replace, zl_setfield, zl_setglobal};
use crate::state::RawState;
use std::ffi::{CStr, c_int};

/// Struct to build Lua module.
pub struct ModuleBuilder<'p, P, N>
where
    P: Frame,
    N: AsRef<CStr>,
{
    parent: &'p mut P,
    name: N,
    has_value: bool,
}

impl<'p, P, N> ModuleBuilder<'p, P, N>
where
    P: Frame,
    N: AsRef<CStr>,
{
    /// # Safety
    /// Top of the stack must be a module table.
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'p mut P, name: N) -> Self {
        Self {
            parent,
            name,
            has_value: false,
        }
    }
}

impl<P, N> Drop for ModuleBuilder<'_, P, N>
where
    P: Frame,
    N: AsRef<CStr>,
{
    #[inline(always)]
    fn drop(&mut self) {
        if self.has_value {
            let name = self.name.as_ref().as_ptr();

            unsafe { zl_pushvalue(self.parent.state(), -1) };
            unsafe { zl_setfield(self.parent.state(), -3, name) };
            unsafe { zl_setglobal(self.parent.state(), name) };
        }

        unsafe { zl_pop(self.parent.state(), 1) };
    }
}

impl<P, N> RawState for ModuleBuilder<'_, P, N>
where
    P: Frame,
    N: AsRef<CStr>,
{
    #[inline(always)]
    fn state(&mut self) -> *mut lua_State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        let excess = n - 1;

        if excess > 0 {
            unsafe { zl_pop(self.state(), excess) };
        }

        if self.has_value {
            unsafe { zl_replace(self.state(), -2) };
        }

        self.has_value = true;
    }
}
