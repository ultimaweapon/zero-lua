use crate::ffi::{lua_State, zl_gettop, zl_touserdata, zl_upvalueindex};
use crate::{Context, Error, NonYieldable};
use std::ffi::c_int;

pub unsafe extern "C-unwind" fn invoker<F>(#[allow(non_snake_case)] L: *mut lua_State) -> c_int
where
    F: Fn(&mut Context<NonYieldable>) -> Result<(), Error> + 'static,
{
    let args = unsafe { zl_gettop(L) };
    let mut cx = unsafe { Context::new(NonYieldable::new(L), args) };
    let cb = if size_of::<F>() == 0 {
        std::ptr::dangling::<F>()
    } else {
        let cb = unsafe { zl_upvalueindex(1) };

        unsafe { zl_touserdata(L, cb).cast::<F>().cast_const() }
    };

    match unsafe { (*cb)(&mut cx) } {
        Ok(_) => cx.into_results(),
        Err(e) => cx.raise(e),
    }
}
