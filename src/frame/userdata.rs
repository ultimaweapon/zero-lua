use crate::ffi::{lua_State, zl_getfield, zl_globalmetatable, zl_pop, zl_touserdata};
use crate::{Type, UserType};
use std::any::{TypeId, type_name};
use std::ffi::c_int;

pub unsafe extern "C-unwind" fn finalizer<T>(#[allow(non_snake_case)] L: *mut lua_State) -> c_int
where
    T: 'static,
{
    let ptr = unsafe { zl_touserdata(L, 1).cast::<T>() };
    unsafe { std::ptr::drop_in_place(ptr) };
    0
}

/// # Panics
/// If `T` is not registered.
#[inline(never)]
pub unsafe fn push_metatable<T: UserType>(#[allow(non_snake_case)] L: *mut lua_State) {
    // Get metatable.
    match unsafe { zl_globalmetatable(L, T::name().as_ptr()) } {
        Type::Nil => panic!("{} is not registered", type_name::<T>()),
        Type::Table => (),
        _ => unreachable!(),
    }

    // SAFETY: Checking field type does not really give us 100% safe. The only cases
    // "typeid" is not our value are either:
    //
    // 1. Our user use lua_State wrong.
    // 2. We screw ourself.
    //
    // The first case required unsafe code and the second case is our own bug.
    unsafe { zl_getfield(L, -1, c"typeid".as_ptr()) };

    // SAFETY: TypeId is Copy.
    let ud = unsafe { zl_touserdata(L, -1) };
    let id = unsafe { ud.cast::<TypeId>().read_unaligned() };

    unsafe { zl_pop(L, 1) };

    if id != TypeId::of::<T>() {
        panic!("{} is not registered", type_name::<T>())
    }
}
