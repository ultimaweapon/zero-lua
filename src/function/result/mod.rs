use crate::ffi::{zl_isnil, zl_tointegerx, zl_type};
use crate::{Frame, Type};
use std::ffi::c_int;

/// Encapsulates function results on the top of Lua stack.
///
/// This encapsulates the results from a call with `LUA_MULTRET`.
pub struct Ret<'a, P: Frame> {
    parent: &'a mut P,
    len: c_int,
}

impl<'a, P: Frame> Ret<'a, P> {
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'a mut P, len: c_int) -> Self {
        Self { parent, len }
    }

    #[inline(always)]
    pub fn len(&self) -> c_int {
        self.len
    }

    /// `n` is one-based the same as function arguments.
    ///
    /// # Panics
    /// If `n` is less than 1 or geater than [len](Self::len()).
    #[inline(always)]
    pub fn to_nil(&mut self, n: c_int) -> Option<()> {
        unsafe { zl_isnil(self.parent.state().get(), self.index(n)).then_some(()) }
    }

    /// `n` is one-based the same as function arguments.
    ///
    /// # Panics
    /// If `n` is less than 1 or geater than [len](Self::len()).
    #[inline(always)]
    pub fn to_int(&mut self, n: c_int) -> Option<i64> {
        let mut ok = 0;
        let val = unsafe { zl_tointegerx(self.parent.state().get(), self.index(n), &mut ok) };

        if ok == 0 { None } else { Some(val) }
    }

    /// `n` is one-based the same as function arguments.
    ///
    /// # Panics
    /// If `n` is less than 1 or geater than [len](Self::len()).
    #[inline(always)]
    pub fn to_type(&mut self, n: c_int) -> Type {
        unsafe { zl_type(self.parent.state().get(), self.index(n)) }
    }

    #[inline(always)]
    fn index(&self, n: c_int) -> c_int {
        assert!(n > 0);

        self.len
            .checked_sub(n)
            .filter(|&v| v >= 0)
            .map(|v| -(v + 1))
            .unwrap()
    }
}

impl<'a, P: Frame> Drop for Ret<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.len > 0 {
            unsafe { self.parent.release_values(self.len) };
        }
    }
}
