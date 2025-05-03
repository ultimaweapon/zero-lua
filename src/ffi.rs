use crate::Type;
use std::ffi::{c_char, c_int, c_void};

pub const LUA_OK: c_int = 0;
pub const LUA_YIELD: c_int = 1;

pub const LUA_MULTRET: c_int = -1;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct lua_State([u8; 0]);

unsafe extern "C-unwind" {
    pub static ZL_REGISTRYINDEX: c_int;

    pub safe fn zl_newstate() -> *mut lua_State;
    pub fn zl_close(L: *mut lua_State);
    pub fn zl_atpanic(L: *mut lua_State, panicf: Option<extern "C" fn(*mut lua_State) -> c_int>);
    pub fn zl_require_base(L: *mut lua_State, global: bool);
    pub fn zl_require_coroutine(L: *mut lua_State, global: bool);
    pub fn zl_require_io(L: *mut lua_State, global: bool);
    pub fn zl_require_math(L: *mut lua_State, global: bool);
    pub fn zl_require_os(L: *mut lua_State, global: bool);
    pub fn zl_require_string(L: *mut lua_State, global: bool);
    pub fn zl_require_table(L: *mut lua_State, global: bool);
    pub fn zl_require_utf8(L: *mut lua_State, global: bool);
    pub fn zl_load(
        L: *mut lua_State,
        name: *const c_char,
        chunk: *const c_char,
        len: usize,
        mode: *const c_char,
    ) -> bool;
    pub fn zl_pcall(L: *mut lua_State, nargs: c_int, nresults: c_int, msgh: c_int) -> bool;
    pub fn zl_checkstack(L: *mut lua_State, n: c_int);
    pub fn zl_pushnil(L: *mut lua_State);
    pub fn zl_pushboolean(L: *mut lua_State, b: bool);
    pub fn zl_pushlstring(L: *mut lua_State, s: *const c_char, len: usize) -> *const c_char;
    pub fn zl_pushlightuserdata(L: *mut lua_State, p: *mut c_void);
    pub fn zl_pushcclosure(
        L: *mut lua_State,
        fp: unsafe extern "C-unwind" fn(*mut lua_State) -> c_int,
        n: c_int,
    );
    pub fn zl_gettop(L: *mut lua_State) -> c_int;
    pub fn zl_checklstring(L: *mut lua_State, arg: c_int, l: *mut usize) -> *const c_char;
    pub fn zl_typeerror(L: *mut lua_State, arg: c_int, tname: *const c_char) -> !;
    pub fn zl_argerror(L: *mut lua_State, arg: c_int, extramsg: *const c_char) -> !;
    pub fn zl_isnil(L: *mut lua_State, index: c_int) -> bool;
    pub fn zl_istable(L: *mut lua_State, index: c_int) -> bool;
    pub fn zl_tointegerx(L: *mut lua_State, index: c_int, isnum: *mut c_int) -> i64;
    pub fn zl_tolstring(L: *mut lua_State, index: c_int, len: *mut usize) -> *const c_char;
    pub fn zl_touserdata(L: *mut lua_State, index: c_int) -> *mut u8;
    pub fn zl_type(L: *mut lua_State, index: c_int) -> Type;
    pub fn zl_typename(L: *mut lua_State, tp: Type) -> *const c_char;
    pub fn zl_createtable(L: *mut lua_State, narr: c_int, nrec: c_int);
    pub fn zl_ref(L: *mut lua_State, t: c_int) -> c_int;
    pub fn zl_unref(L: *mut lua_State, t: c_int, r#ref: c_int);
    pub fn zl_geti(L: *mut lua_State, index: c_int, i: i64) -> Type;
    pub fn zl_seti(L: *mut lua_State, index: c_int, n: i64);
    pub fn zl_getfield(L: *mut lua_State, index: c_int, k: *const c_char) -> Type;
    pub fn zl_setfield(L: *mut lua_State, index: c_int, k: *const c_char);
    pub fn zl_newuserdatauv(L: *mut lua_State, size: usize, nuvalue: c_int) -> *mut u8;
    pub fn zl_setiuservalue(L: *mut lua_State, index: c_int, n: u16) -> c_int;
    pub fn zl_getiuservalue(L: *mut lua_State, index: c_int, n: u16) -> Type;
    pub fn zl_newmetatable(L: *mut lua_State, tname: *const c_char) -> c_int;
    pub fn zl_globalmetatable(L: *mut lua_State, tname: *const c_char) -> Type;
    pub fn zl_setmetatable(L: *mut lua_State, index: c_int);
    pub fn zl_getmetatable(L: *mut lua_State, index: c_int) -> c_int;
    pub fn zl_getmetafield(L: *mut lua_State, obj: c_int, e: *const c_char) -> Type;
    pub fn zl_upvalueindex(i: c_int) -> c_int;
    pub fn zl_setglobal(L: *mut lua_State, name: *const c_char);
    pub fn zl_replace(L: *mut lua_State, index: c_int);
    pub fn zl_pop(L: *mut lua_State, n: c_int);
    pub fn zl_error(L: *mut lua_State, msg: *const c_char) -> !;
    pub fn zl_getextraspace(L: *mut lua_State) -> *mut *mut ();
    pub fn zl_newthread(L: *mut lua_State) -> *mut lua_State;
    pub fn zl_resume(
        L: *mut lua_State,
        from: *mut lua_State,
        nargs: c_int,
        nresults: &mut c_int,
    ) -> c_int;
    pub fn zl_yieldk(
        L: *mut lua_State,
        nresults: c_int,
        ctx: isize,
        k: unsafe extern "C-unwind" fn(L: *mut lua_State, status: c_int, ctx: isize) -> c_int,
    ) -> !;
}
