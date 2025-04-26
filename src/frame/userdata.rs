use crate::ffi::{lua_State, zl_touserdata};
use std::ffi::c_int;

pub unsafe extern "C-unwind" fn finalizer<T>(#[allow(non_snake_case)] L: *mut lua_State) -> c_int
where
    T: 'static,
{
    let ptr = unsafe { zl_touserdata(L, 1).cast::<T>() };
    unsafe { std::ptr::drop_in_place(ptr) };
    0
}
