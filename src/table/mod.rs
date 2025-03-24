pub use self::borrowed::*;
pub use self::setter::*;

use crate::ffi::{engine_setfield, lua_State, lua54_getfield, lua54_geti, lua54_seti};
use crate::{Frame, Type};
use std::ffi::{CStr, c_int};

mod borrowed;
mod setter;

/// Encapsulates an owned table in the stack.
pub struct Table<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Table<'a, P> {
    /// # Safety
    /// Top of the stack must be a table.
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }

    /// Calling this method without pushing a value to [`TableSetter`] does nothing.
    ///
    /// Note that the returned [`TableSetter`] only keep the last pushed value.
    pub fn set<K: TableKey>(&mut self, key: K) -> TableSetter<'_, 'a, P, K> {
        unsafe { TableSetter::new(self, key) }
    }
}

impl<'a, P: Frame> Drop for Table<'a, P> {
    fn drop(&mut self) {
        unsafe { self.0.release_items(1) };
    }
}

/// Key to lookup for a value in the table.
pub trait TableKey {
    unsafe fn get(&self, state: *mut lua_State, table: c_int) -> Type;
    unsafe fn set(&self, state: *mut lua_State, table: c_int);
}

impl TableKey for i64 {
    unsafe fn get(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_geti(state, table, *self) }
    }

    unsafe fn set(&self, state: *mut lua_State, table: c_int) {
        unsafe { lua54_seti(state, table, *self) };
    }
}

impl TableKey for &CStr {
    unsafe fn get(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_getfield(state, table, self.as_ptr()) }
    }

    unsafe fn set(&self, state: *mut lua_State, table: c_int) {
        unsafe { engine_setfield(state, table, self.as_ptr()) };
    }
}
