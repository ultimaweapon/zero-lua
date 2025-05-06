use crate::Frame;
use crate::ffi::{zl_pop, zl_pushvalue, zl_replace, zl_setfield, zl_setglobal};
use crate::state::FrameState;
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

            unsafe { zl_pushvalue(self.parent.state().get(), -1) };
            unsafe { zl_setfield(self.parent.state().get(), -3, name) };
            unsafe { zl_setglobal(self.parent.state().get(), name) };
        }

        unsafe { zl_pop(self.parent.state().get(), 1) };
    }
}

impl<P, N> FrameState for ModuleBuilder<'_, P, N>
where
    P: Frame,
    N: AsRef<CStr>,
{
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        let excess = n - 1;

        if excess > 0 {
            unsafe { zl_pop(self.state().get(), excess) };
        }

        if self.has_value {
            unsafe { zl_replace(self.state().get(), -2) };
        }

        self.has_value = true;
    }
}
