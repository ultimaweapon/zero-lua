use crate::ffi::{engine_pop, engine_touserdata, lua_State, lua54_getfield, zl_globalmetatable};
use crate::{Type, UserData};
use std::any::{TypeId, type_name};

/// # Safety
/// Lua stack must have at least 2 slots available.
///
/// # Panics
/// If `T` is not registered.
#[inline(never)]
pub unsafe fn push_metatable<T: UserData>(#[allow(non_snake_case)] L: *mut lua_State) {
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
    unsafe { lua54_getfield(L, -1, c"typeid".as_ptr()) };

    // SAFETY: TypeId is Copy.
    let ud = unsafe { engine_touserdata(L, -1) };
    let id = unsafe { ud.cast::<TypeId>().read_unaligned() };

    unsafe { engine_pop(L, 1) };

    if id != TypeId::of::<T>() {
        panic!("{} is not registered", type_name::<T>())
    }
}
