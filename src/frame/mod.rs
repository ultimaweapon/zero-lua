use crate::ffi::{
    engine_checkstack, engine_createtable, engine_newuserdatauv, engine_pop, engine_pushcclosure,
    engine_pushnil, engine_setfield, engine_setmetatable, engine_touserdata, engine_upvalueindex,
    lua_State, lua54_getfield, zl_load, zl_newmetatable, zl_pushlstring, zl_require_os,
};
use crate::{
    Context, Error, Function, GlobalSetter, Nil, Str, Table, UserData, UserValue, is_boxed,
};
use std::any::TypeId;
use std::ffi::{CStr, c_int};
use std::mem::ManuallyDrop;
use std::panic::UnwindSafe;
use std::path::Path;

/// Frame in a Lua stack.
///
/// Most methods in this trait can raise a C++ exception. When calling outside Lua runtime it will
/// cause the process to terminate the same as Rust panic. Inside Lua runtime it will report as Lua
/// error. Usually you don't need to worry about this **unless** you use any type that does not
/// implement a proper RAII.
pub trait Frame: Sized {
    fn require_os(&mut self) -> Table<Self> {
        // SAFETY: 3 is maximum stack size used by engine_require_os.
        unsafe { engine_checkstack(self.state(), 3) };
        unsafe { zl_require_os(self.state()) };

        unsafe { Table::new(self) }
    }

    fn set_global<N: AsRef<CStr>>(&mut self, name: N) -> GlobalSetter<Self, N> {
        GlobalSetter::new(self, name)
    }

    /// This method will load the whole content of `file` into memory before passing to Lua.
    fn load_file(
        &mut self,
        file: impl AsRef<Path>,
    ) -> Result<Result<Function<Self>, Str<Self>>, std::io::Error> {
        // SAFETY: engine_load return either error object or a chunk.
        unsafe { engine_checkstack(self.state(), 1) };

        // Read file.
        let file = file.as_ref();
        let data = std::fs::read(file)?;

        // Get chunk name.
        let file = file.to_string_lossy();
        let mut name = String::with_capacity(1 + file.len() + 1);

        name.push('@');
        name.push_str(&file);
        name.push('\0');

        // Load.
        let name = name.as_ptr().cast();
        let r = match unsafe { zl_load(self.state(), name, data.as_ptr().cast(), data.len()) } {
            true => Ok(unsafe { Function::new(self) }),
            false => Err(unsafe { Str::new(self) }),
        };

        Ok(r)
    }

    fn push_nil(&mut self) -> Nil<Self> {
        unsafe { engine_checkstack(self.state(), 1) };
        unsafe { engine_pushnil(self.state()) };

        unsafe { Nil::new(self) }
    }

    fn push_str(&mut self, s: impl AsRef<[u8]>) -> Str<Self> {
        let s = s.as_ref();

        unsafe { engine_checkstack(self.state(), 1) };
        unsafe { zl_pushlstring(self.state(), s.as_ptr().cast(), s.len()) };

        unsafe { Str::new(self) }
    }

    /// See [`Context`] for how to return some values to Lua.
    fn push_fn<F>(&mut self, f: F) -> Function<Self>
    where
        F: Fn(&mut Context) -> Result<(), Error> + UnwindSafe + 'static,
    {
        // SAFETY: 3 is maximum items we pushed here.
        unsafe { engine_checkstack(self.state(), 3) };

        if align_of::<F>() <= align_of::<*mut ()>() {
            // Move Rust function to Lua user data.
            let ptr = unsafe { engine_newuserdatauv(self.state(), size_of::<F>(), 0) };

            unsafe { ptr.cast::<F>().write(f) };

            // Set finalizer.
            if std::mem::needs_drop::<F>() {
                unsafe { engine_createtable(self.state(), 0, 1) };
                unsafe { engine_pushcclosure(self.state(), finalizer::<F>, 0) };
                unsafe { engine_setfield(self.state(), -2, c"__gc".as_ptr()) };
                unsafe { engine_setmetatable(self.state(), -1) };
            }

            // Push invoker.
            unsafe { engine_pushcclosure(self.state(), invoker::<F>, 1) };
        } else {
            // Move Rust function to Lua user data.
            let ptr = unsafe { engine_newuserdatauv(self.state(), size_of::<Box<F>>(), 0) };

            unsafe { ptr.cast::<Box<F>>().write(f.into()) };

            // Set finalizer.
            unsafe { engine_createtable(self.state(), 0, 1) };
            unsafe { engine_pushcclosure(self.state(), finalizer::<Box<F>>, 0) };
            unsafe { engine_setfield(self.state(), -2, c"__gc".as_ptr()) };
            unsafe { engine_setmetatable(self.state(), -1) };

            // Push invoker.
            unsafe { engine_pushcclosure(self.state(), invoker::<Box<F>>, 1) };
        }

        unsafe { Function::new(self) }
    }

    fn push_table(&mut self, narr: u16, nrec: u16) -> Table<Self> {
        unsafe { engine_checkstack(self.state(), 1) };
        unsafe { engine_createtable(self.state(), narr.into(), nrec.into()) };

        unsafe { Table::new(self) }
    }

    /// # Panics
    /// If [`UserData::name()`] on `T` is duplicated with the other type.
    fn push_ud<T: UserData>(&mut self, v: T) -> UserValue<Self> {
        // SAFETY: Maximum pushed from luaL_newmetatable is 2.
        unsafe { engine_checkstack(self.state(), 3) };

        if is_boxed::<T>() {
            let ptr = unsafe { engine_newuserdatauv(self.state(), size_of::<Box<T>>(), 0) };

            unsafe { ptr.cast::<Box<T>>().write(v.into()) };
        } else {
            let ptr = unsafe { engine_newuserdatauv(self.state(), size_of::<T>(), 0) };

            unsafe { ptr.cast::<T>().write(v) };
        }

        unsafe { push_metatable::<Self, T>(self) };
        unsafe { engine_setmetatable(self.state(), -1) };

        unsafe { UserValue::new(self) }
    }

    /// Returns a `lua_State` this frame belong to.
    ///
    /// This is a low-level method. Using the returned `lua_State` incorrectly will violate safety
    /// guarantee of this crate. This does not mark as `unsafe` because invoke this method is safe
    /// but using the returned pointer required unsafe code.
    fn state(&self) -> *mut lua_State;

    /// # Safety
    /// `n` must be greater than zero and `n` values on the top of stack must be owned by the
    /// caller.
    unsafe fn release_values(&mut self, n: c_int);
}

#[inline(never)]
unsafe fn push_metatable<F: Frame, T: UserData>(f: &mut F) {
    // Check if exists.
    let id = TypeId::of::<T>();

    if unsafe { zl_newmetatable(f.state(), T::name().as_ptr()) == 0 } {
        // SAFETY: Checking field type does not really give us 100% safe. The only cases
        // "typeid" is not our value are either:
        //
        // 1. Our user use lua_State wrong.
        // 2. We screw ourself.
        //
        // The first case required unsafe code and the second case is our own bug.
        unsafe { lua54_getfield(f.state(), -1, c"typeid".as_ptr()) };

        // SAFETY: TypeId is Copy.
        let ud = unsafe { engine_touserdata(f.state(), -1) };

        assert_eq!(unsafe { ud.cast::<TypeId>().read_unaligned() }, id);

        unsafe { engine_pop(f.state(), 1) };
        return;
    }

    // Setup metatable.
    T::setup_metatable(&mut ManuallyDrop::new(unsafe { Table::new(f) }));

    // Set "typeid".
    let ud = unsafe { engine_newuserdatauv(f.state(), size_of::<TypeId>(), 0) };

    unsafe { ud.cast::<TypeId>().write_unaligned(id) };
    unsafe { engine_setfield(f.state(), -2, c"typeid".as_ptr()) };

    // Set finalizer.
    if is_boxed::<T>() {
        unsafe { engine_pushcclosure(f.state(), finalizer::<Box<T>>, 0) };
        unsafe { engine_setfield(f.state(), -2, c"__gc".as_ptr()) };
    } else if std::mem::needs_drop::<T>() {
        unsafe { engine_pushcclosure(f.state(), finalizer::<T>, 0) };
        unsafe { engine_setfield(f.state(), -2, c"__gc".as_ptr()) };
    }
}

unsafe extern "C-unwind" fn invoker<F>(#[allow(non_snake_case)] L: *mut lua_State) -> c_int
where
    F: Fn(&mut Context) -> Result<(), Error> + UnwindSafe + 'static,
{
    let mut cx = unsafe { Context::new(L) };
    let cb = unsafe { engine_upvalueindex(1) };
    let cb = unsafe { engine_touserdata(L, cb).cast::<F>().cast_const() };

    match unsafe { (*cb)(&mut cx) } {
        Ok(_) => cx.into_results(),
        Err(e) => cx.raise(e),
    }
}

unsafe extern "C-unwind" fn finalizer<T>(#[allow(non_snake_case)] L: *mut lua_State) -> c_int {
    let ptr = unsafe { engine_touserdata(L, 1).cast::<T>() };
    unsafe { std::ptr::drop_in_place(ptr) };
    0
}
