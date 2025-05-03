pub use self::state::*;

use crate::ffi::{
    zl_argerror, zl_checklstring, zl_error, zl_getfield, zl_getmetatable, zl_isnil, zl_istable,
    zl_pop, zl_tolstring, zl_touserdata, zl_typeerror,
};
use crate::state::FrameState;
use crate::{BorrowedTable, BorrowedUd, Error, ErrorKind, PositiveInt, UserType, is_boxed};
use std::any::TypeId;
use std::ffi::c_int;
use std::marker::PhantomData;

mod state;

/// Encapsulates a `lua_State` passed to `lua_CFunction`.
///
/// All values pushed directly to this struct will become function results.
pub struct Context<'a, S = NonYieldable> {
    state: S,
    args: c_int,
    ret: c_int,
    phantom: PhantomData<&'a ()>,
}

impl<'a, S: LocalState> Context<'a, S> {
    #[inline(always)]
    pub(crate) unsafe fn new(state: S, args: c_int) -> Self {
        Self {
            state,
            args,
            ret: 0,
            phantom: PhantomData,
        }
    }

    /// Returns number of arguments for the current function. This also the index of the last
    /// argument.
    #[inline(always)]
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
            unsafe { zl_isnil(self.state.get(), n) }
        } else {
            true
        }
    }

    /// Get UTF-8 string argument or raise a Lua error if the argument cannot convert to a UTF-8
    /// string.
    ///
    /// This method always raise a Lua error if `n` is not a function argument.
    pub fn to_str(&self, n: PositiveInt) -> &'a str {
        if n > self.args {
            // engine_checkstring require a valid index so we need to emulate its behavior in this
            // case.
            self.arg_out_of_bound(n, b"string");
        }

        // SAFETY: luaL_checklstring never return null.
        let mut l = 0;
        let v = unsafe { zl_checklstring(self.state.get(), n.get(), &mut l) };
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
    pub fn try_str(&self, n: PositiveInt) -> Option<&'a str> {
        if n > self.args {
            return None;
        }

        // Get value.
        let mut l = 0;
        let v = unsafe { zl_tolstring(self.state.get(), n.get(), &mut l) };

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
    #[inline(always)]
    pub fn to_table(&mut self, n: PositiveInt) -> BorrowedTable<Self> {
        if n > self.args {
            // lua_istable require a valid index so we need to emulate its behavior in this case.
            self.arg_out_of_bound(n, b"table");
        } else if !unsafe { zl_istable(self.state.get(), n.get()) } {
            unsafe { zl_typeerror(self.state.get(), n.get(), c"table".as_ptr()) };
        }

        unsafe { BorrowedTable::new(self, n) }
    }

    /// Get table argument or returns [`None`] if the argument is not a table.
    ///
    /// This method always return [`None`] if `n` is not a function argument.
    #[inline(always)]
    pub fn try_table(&mut self, n: PositiveInt) -> Option<BorrowedTable<Self>> {
        if n > self.args || !unsafe { zl_istable(self.state.get(), n.get()) } {
            return None;
        }

        Some(unsafe { BorrowedTable::new(self, n) })
    }

    pub fn to_ud<T: UserType>(&mut self, n: PositiveInt) -> BorrowedUd<'_, 'a, Self, T> {
        if n > self.args {
            // lua_touserdata require a valid index so we need to emulate luaL_checkudata behavior
            // in this case.
            self.arg_out_of_bound(n, T::name().to_bytes());
        }

        // We emulate luaL_checkudata here since we need to get additional field from metatable.
        let ptr = unsafe { zl_touserdata(self.state.get(), n.get()).cast_const() };

        if ptr.is_null() || unsafe { zl_getmetatable(self.state.get(), n.get()) == 0 } {
            unsafe { zl_typeerror(self.state.get(), n.get(), T::name().as_ptr()) };
        }

        unsafe { zl_getfield(self.state.get(), -1, c"typeid".as_ptr()) };

        // SAFETY: TypeId is Copy.
        let id = TypeId::of::<T>();
        let ud = unsafe { zl_touserdata(self.state.get(), -1) };
        let ok = unsafe { !ud.is_null() && ud.cast::<TypeId>().read_unaligned() == id };

        unsafe { zl_pop(self.state.get(), 2) };

        // Get pointer to UD.
        let ud = if !ok {
            unsafe { zl_typeerror(self.state.get(), n.get(), T::name().as_ptr()) };
        } else if is_boxed::<T>() {
            unsafe { (*ptr.cast::<Box<T>>()).as_ref() }
        } else {
            unsafe { &*ptr.cast::<T>() }
        };

        unsafe { BorrowedUd::new(self, n, ud) }
    }

    pub(crate) fn into_results(self) -> c_int {
        self.ret
    }

    #[inline(never)]
    pub(crate) fn raise(&self, e: Error) -> ! {
        let (n, e) = match e.into() {
            // SAFETY: n only used to format the message.
            ErrorKind::Arg(n, e) => unsafe {
                zl_argerror(self.state.get(), n.get(), e.as_ptr().cast())
            },
            ErrorKind::ArgType(n, e) => (n, e),
            ErrorKind::Other(e) => unsafe { zl_error(self.state.get(), e.as_ptr().cast()) },
        };

        if n <= self.args {
            unsafe { zl_typeerror(self.state.get(), n.get(), e.as_ptr().cast()) };
        } else {
            // lua54_typeerror require index to be valid so we need to emulate its behavior in this
            // case.
            self.arg_out_of_bound(n, &e[..(e.len() - 1)]);
        }
    }

    #[inline(never)]
    fn arg_out_of_bound(&self, n: PositiveInt, expect: &[u8]) -> ! {
        let s = b" expected, got nil";
        let mut m = Vec::with_capacity(expect.len() + s.len() + 1);

        m.extend_from_slice(expect);
        m.extend_from_slice(s);
        m.push(0);

        unsafe { zl_argerror(self.state.get(), n.get(), m.as_ptr().cast()) };
    }
}

impl<S: LocalState> FrameState for Context<'_, S> {
    type State = S;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        &mut self.state
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        // SAFETY: We don't need to check for overflow here since we should get hit by a stack
        // overflow before c_int overflow.
        self.ret += n;
    }
}
