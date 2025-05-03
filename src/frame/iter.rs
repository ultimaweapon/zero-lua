use crate::convert::IntoLua;
use crate::ffi::{lua_State, zl_pushnil, zl_touserdata};
use crate::{Context, NonYieldable};
use std::ffi::c_int;
use std::iter::FusedIterator;

pub unsafe extern "C-unwind" fn next<T>(#[allow(non_snake_case)] L: *mut lua_State) -> c_int
where
    T: FusedIterator<Item: IntoLua>,
{
    // SAFETY: We don't allow the user to get arbitrary userdata.
    let iter = unsafe { &mut *zl_touserdata(L, 1).cast::<T>() };
    let mut cx = unsafe { Context::new(NonYieldable::new(L), 2) };

    match iter.next() {
        Some(v) => drop(v.into_lua(&mut cx)),
        None => {
            for _ in 0..T::Item::N.get() {
                unsafe { zl_pushnil(L) };
            }
        }
    }

    cx.into_results()
}
