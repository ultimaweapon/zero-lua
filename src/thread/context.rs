use crate::ffi::{
    engine_argerror, engine_error, engine_gettop, engine_isnil, lua_State, lua54_istable,
    lua54_typeerror, zl_checklstring, zl_tolstring,
};
use crate::{BorrowedTable, Error, ErrorKind, Frame};
use std::ffi::c_int;

/// Encapsulates a `lua_State` passed to `lua_CFunction`.
///
/// All values pushed directly to this struct will become function results.
pub struct Context {
    state: *mut lua_State,
    args: c_int,
    ret: c_int,
}

impl Context {
    pub(crate) unsafe fn new(state: *mut lua_State) -> Self {
        let args = unsafe { engine_gettop(state) };

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

    /// Get UTF-8 string argument or raise a Lua error if the argument cannot convert to a UTF-8
    /// string.
    ///
    /// This method always raise a Lua error if `n` is not a function argument.
    ///
    /// # Panics
    /// If `n` is zero or negative.
    pub fn to_str<'a, 'b: 'a>(&'b self, n: c_int) -> &'a str {
        assert!(n > 0);

        if n > self.args {
            // engine_checkstring require a valid index so we need to emulate its behavior in this
            // case.
            self.arg_out_of_bound(n, b"string");
        }

        // SAFETY: luaL_checklstring never return null.
        let mut l = 0;
        let v = unsafe { zl_checklstring(self.state, n, &mut l) };
        let v = unsafe { std::slice::from_raw_parts(v.cast(), l) };

        match std::str::from_utf8(v) {
            Ok(v) => v,
            Err(e) => self.raise(Error::arg_from_std(n, e)),
        }
    }

    /// Get UTF-8 string argument or raise a Lua error if the argument is a string but not valid
    /// UTF-8.
    ///
    /// This method return [`None`] if the argument is not a string or `n` is not a function
    /// argument.
    ///
    /// # Panics
    /// If `n` is zero or negative.
    pub fn try_str<'a, 'b: 'a>(&'b self, n: c_int) -> Option<&'a str> {
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

        // Check if UTF-8.
        let v = unsafe { std::slice::from_raw_parts(v.cast(), l) };

        match std::str::from_utf8(v) {
            Ok(v) => Some(v),
            Err(e) => self.raise(Error::arg_from_std(n, e)),
        }
    }

    /// Get table argument or raise a Lua error if the argument is not a table.
    ///
    /// This method always raise a Lua error if `n` is not a function argument.
    ///
    /// # Panics
    /// If `n` is zero or negative.
    pub fn to_table(&mut self, n: c_int) -> BorrowedTable<Self> {
        assert!(n > 0);

        if n > self.args {
            // lua_istable require a valid index so we need to emulate its behavior in this case.
            self.arg_out_of_bound(n, b"table");
        } else if !unsafe { lua54_istable(self.state, n) } {
            unsafe { lua54_typeerror(self.state, n, c"table".as_ptr()) };
        }

        unsafe { BorrowedTable::new(self, n) }
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

    #[inline(never)]
    pub(crate) fn raise(&self, e: Error) -> ! {
        let (n, e) = match e.into() {
            // SAFETY: n only used to format the message.
            ErrorKind::Arg(n, e) => unsafe { engine_argerror(self.state, n, e.as_ptr().cast()) },
            ErrorKind::ArgType(n, e) => (n, e),
            ErrorKind::Other(e) => unsafe { engine_error(self.state, e.as_ptr().cast()) },
        };

        if n <= self.args {
            // SAFETY: n is positive.
            unsafe { lua54_typeerror(self.state, n, e.as_ptr().cast()) };
        } else {
            // lua54_typeerror require index to be valid so we need to emulate its behavior in this
            // case.
            self.arg_out_of_bound(n, &e[..(e.len() - 1)]);
        }
    }

    #[inline(never)]
    fn arg_out_of_bound(&self, n: c_int, expect: &[u8]) -> ! {
        let s = b" expected, got nil";
        let mut m = Vec::with_capacity(expect.len() + s.len() + 1);

        m.extend_from_slice(expect);
        m.extend_from_slice(s);
        m.push(0);

        unsafe { engine_argerror(self.state, n, m.as_ptr().cast()) };
    }
}

impl Frame for Context {
    fn state(&self) -> *mut lua_State {
        self.state
    }

    unsafe fn release_values(&mut self, n: c_int) {
        // SAFETY: We don't need to check for overflow here since we should get hit by a stack
        // overflow before c_int overflow.
        self.ret += n;
    }
}
