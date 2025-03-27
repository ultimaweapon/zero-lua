use crate::ffi::{
    engine_argerror, engine_isnil, lua_State, lua54_istable, zl_checklstring, zl_tolstring,
};
use crate::{BorrowedTable, Frame};
use std::ffi::c_int;

/// Encapsulates a `lua_State` passed to `lua_CFunction`.
///
/// All values pushed directly to this struct will become function results.
pub struct FuncState {
    state: *mut lua_State,
    args: c_int,
    ret: c_int,
}

impl FuncState {
    pub(crate) unsafe fn new(state: *mut lua_State, args: c_int) -> Self {
        Self {
            state,
            args,
            ret: 0,
        }
    }

    /// Returns number of arguments for the current function. This also the index of the last
    /// argument.
    pub fn args(&self) -> c_int {
        self.args
    }

    /// Checks if argument is `nil`.
    ///
    /// This method always return `true` if `n` is not a function argument.
    ///
    /// # Panics
    /// If `n` is zero or negative.
    pub fn is_nil(&self, n: c_int) -> bool {
        assert!(n > 0);

        if n <= self.args {
            unsafe { engine_isnil(self.state, n) }
        } else {
            true
        }
    }

    /// Get string argument or raise a Lua error is the argument cannot convert to a string.
    ///
    /// The returned slice will **not** contain the trailing NUL terminator. However, it is
    /// guarantee there is a NUL past the end.
    ///
    /// This method always raise a Lua error if `n` is not a function argument.
    ///
    /// # Panics
    /// If `n` is zero or negative.
    pub fn to_string<'a, 'b: 'a>(&'b self, n: c_int) -> &'a [u8] {
        assert!(n > 0);

        if n > self.args {
            // engine_checkstring require a valid index so we need to emulate its behavior in this
            // case.
            self.arg_out_of_bound(n, b"string");
        }

        // SAFETY: luaL_checklstring never return null.
        let mut l = 0;
        let s = unsafe { zl_checklstring(self.state, n, &mut l) };

        unsafe { std::slice::from_raw_parts(s.cast(), l) }
    }

    /// Get string argument or returns [`None`] if the argument cannot convert to a string.
    ///
    /// The returned slice will **not** contain the trailing NUL terminator. However, it is
    /// guarantee there is a NUL past the end.
    ///
    /// This method always return [`None`] if `n` is not a function argument.
    ///
    /// # Panics
    /// If `n` is zero or negative.
    pub fn try_string<'a, 'b: 'a>(&'b self, n: c_int) -> Option<&'a [u8]> {
        assert!(n > 0);

        if n > self.args {
            return None;
        }

        // Get value.
        let mut l = 0;
        let v = unsafe { zl_tolstring(self.state, n, &mut l) };

        if v.is_null() {
            return None;
        }

        Some(unsafe { std::slice::from_raw_parts(v.cast(), l) })
    }

    /// Get table argument or returns [`None`] if the argument is not a table.
    ///
    /// This method always return [`None`] if `n` is not a function argument.
    ///
    /// # Panics
    /// If `n` is zero or negative.
    pub fn try_table(&mut self, n: c_int) -> Option<BorrowedTable<Self>> {
        assert!(n > 0);

        if n > self.args || !unsafe { lua54_istable(self.state, n) } {
            return None;
        }

        Some(unsafe { BorrowedTable::new(self, n) })
    }

    pub(crate) fn into_results(self) -> c_int {
        self.ret
    }

    pub(crate) fn arg_out_of_bound(&self, n: c_int, expect: &[u8]) -> ! {
        let s = b" expected, got nil";
        let mut m = Vec::with_capacity(expect.len() + s.len() + 1);

        m.extend_from_slice(expect);
        m.extend_from_slice(s);
        m.push(0);

        unsafe { engine_argerror(self.state, n, m.as_ptr().cast()) };
    }
}

impl Frame for FuncState {
    fn state(&self) -> *mut lua_State {
        self.state
    }

    unsafe fn release_values(&mut self, n: c_int) {
        // SAFETY: We don't need to check for overflow here since we should get hit by a stack
        // overflow before c_int overflow.
        self.ret += n;
    }
}
