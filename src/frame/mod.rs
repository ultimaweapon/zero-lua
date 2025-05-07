pub use self::r#yield::*;

use self::r#async::async_invoker;
use self::function::invoker;
use self::userdata::{finalizer, push_metatable};
use crate::convert::IntoLua;
use crate::ffi::{
    ZL_LOADED_TABLE, ZL_REGISTRYINDEX, zl_checkstack, zl_createtable, zl_getfield, zl_getsubtable,
    zl_load, zl_newmetatable, zl_newuserdatauv, zl_pop, zl_pushboolean, zl_pushcclosure,
    zl_pushlstring, zl_pushnil, zl_require_base, zl_require_coroutine, zl_require_io,
    zl_require_math, zl_require_os, zl_require_string, zl_require_table, zl_require_utf8,
    zl_setfield, zl_setmetatable,
};
use crate::state::RawState;
use crate::{
    Bool, ChunkType, Context, Error, Function, GlobalSetter, Iter, ModuleBuilder, Nil,
    NonYieldable, OwnedUd, PositiveInt, Str, TYPE_ID, Table, Type, UserType, Yieldable, is_boxed,
};
use std::any::{TypeId, type_name};
use std::ffi::CStr;
use std::iter::Fuse;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::ptr::null;

mod r#async;
mod function;
mod iter;
mod userdata;
mod r#yield;

/// Virtual frame in a Lua stack.
///
/// Some methods in this trait can raise a Lua error. When calling outside Lua runtime it will
/// trigger Lua panic, which terminate the process.
pub trait Frame: RawState {
    /// Register a type of full userdata.
    ///
    /// See [`Frame::try_register_ud()`] for non-panic version.
    ///
    /// # Errors
    /// If memory is not enough plus any error trigger by [`UserType::setup()`] or
    /// [`UserType::register()`] implemented on `T`.
    ///
    /// # Panics
    /// If the other userdata with the same name as `T` already registered.
    fn register_ud<T: UserType>(&mut self) {
        if !self.try_register_ud::<T>() {
            panic!("{} already registered", T::name().to_string_lossy());
        }
    }

    /// Register a type of full userdata.
    ///
    /// Returns `true` if `T` was successfully registered or `false` if the other userdata with the
    /// same name already registered.
    ///
    /// # Errors
    /// If memory is not enough plus any error trigger by [`UserType::setup()`] or
    /// [`UserType::register()`] implemented on `T`.
    fn try_register_ud<T: UserType>(&mut self) -> bool {
        // Check if exists.
        if unsafe { zl_newmetatable(self.state(), T::name().as_ptr()) == 0 } {
            unsafe { zl_pop(self.state(), 1) };
            return false;
        }

        T::setup(&mut ManuallyDrop::new(unsafe { Table::new(self) }));

        // Check if user supplied typeid.
        match unsafe { zl_getfield(self.state(), -1, TYPE_ID.as_ptr()) } {
            Type::Nil => unsafe { zl_pop(self.state(), 1) },
            _ => unsafe {
                zl_pop(self.state(), 2);

                panic!(
                    "UserType::setup() implementation on {} put a reserved '{}' to metatable",
                    type_name::<T>(),
                    TYPE_ID.to_string_lossy()
                );
            },
        }

        // Check if user supplied __gc.
        match unsafe { zl_getfield(self.state(), -1, c"__gc".as_ptr()) } {
            Type::Nil => unsafe { zl_pop(self.state(), 1) },
            _ => unsafe {
                zl_pop(self.state(), 2);

                panic!(
                    "UserType::setup() implementation on {} put a reserved '__gc' to metatable",
                    type_name::<T>(),
                );
            },
        }

        // Set "typeid".
        let ud = unsafe { zl_newuserdatauv(self.state(), size_of::<TypeId>(), 0) };

        unsafe { ud.cast::<TypeId>().write_unaligned(TypeId::of::<T>()) };
        unsafe { zl_setfield(self.state(), -2, TYPE_ID.as_ptr()) };

        // Set finalizer.
        if is_boxed::<T>() {
            unsafe { zl_pushcclosure(self.state(), finalizer::<Box<T>>, 0) };
            unsafe { zl_setfield(self.state(), -2, c"__gc".as_ptr()) };
        } else if std::mem::needs_drop::<T>() {
            unsafe { zl_pushcclosure(self.state(), finalizer::<T>, 0) };
            unsafe { zl_setfield(self.state(), -2, c"__gc".as_ptr()) };
        }

        unsafe { zl_pop(self.state(), 1) };

        // Add to global.
        T::register(GlobalSetter::new(self, T::name()));

        true
    }

    /// Load [basic library](https://www.lua.org/manual/5.4/manual.html#6.1).
    ///
    /// If this library already loaded this simply return it.
    ///
    /// This use `luaL_requiref` + `luaopen_base` under the hood.
    ///
    /// # Errors
    /// If memory is not enough.
    #[inline(always)]
    fn require_base(&mut self) -> Table<Self> {
        unsafe { zl_require_base(self.state()) };
        unsafe { Table::new(self) }
    }

    /// Load [coroutine library](https://www.lua.org/manual/5.4/manual.html#6.2).
    ///
    /// If this library already loaded this simply return it. Specify `true` for `global` if you
    /// want to put the library itself to global environment otherwise `coroutine` table will not
    /// available to Lua.
    ///
    /// This use `luaL_requiref` + `luaopen_coroutine` under the hood.
    ///
    /// # Errors
    /// If memory is not enough.
    #[inline(always)]
    fn require_coroutine(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_coroutine(self.state(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_io(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_io(self.state(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_math(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_math(self.state(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_os(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_os(self.state(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_string(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_string(self.state(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_table(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_table(self.state(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_utf8(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_utf8(self.state(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn register_module<N: AsRef<CStr>>(&mut self, name: N) -> Option<ModuleBuilder<Self, N>> {
        unsafe { zl_getsubtable(self.state(), ZL_REGISTRYINDEX, ZL_LOADED_TABLE) };

        match unsafe { zl_getfield(self.state(), -1, name.as_ref().as_ptr()) } {
            Type::None => unreachable!(),
            Type::Nil => {
                unsafe { zl_pop(self.state(), 1) };
                Some(unsafe { ModuleBuilder::new(self, name) })
            }
            _ => None,
        }
    }

    #[inline(always)]
    fn set_global<N: AsRef<CStr>>(&mut self, name: N) -> GlobalSetter<Self, N> {
        GlobalSetter::new(self, name)
    }

    /// Load a Lua chunk (AKA Lua code).
    ///
    /// This method use
    /// [luaL_loadbufferx](https://www.lua.org/manual/5.4/manual.html#luaL_loadbufferx) to load the
    /// chunk.
    #[inline(always)]
    fn load(
        &mut self,
        name: Option<&CStr>,
        ty: ChunkType,
        chunk: impl AsRef<[u8]>,
    ) -> Result<Function<Self>, Str<Self>> {
        let name = name.map(|v| v.as_ptr()).unwrap_or(null());
        let chunk = chunk.as_ref();
        let mode = ty.to_c_str();

        match unsafe {
            zl_load(
                self.state(),
                name,
                chunk.as_ptr().cast(),
                chunk.len(),
                mode.as_ptr(),
            )
        } {
            true => Ok(unsafe { Function::new(self) }),
            false => Err(unsafe { Str::new(self) }),
        }
    }

    /// Load a Lua chunk (AKA Lua code) from a file.
    ///
    /// Note that this method will load the whole content of `file` into memory before passing to
    /// Lua.
    fn load_file(
        &mut self,
        file: impl AsRef<Path>,
        ty: ChunkType,
    ) -> Result<Result<Function<Self>, Str<Self>>, std::io::Error> {
        // Read file.
        let file = file.as_ref();
        let data = std::fs::read(file)?;

        // Get chunk name.
        let file = file.to_string_lossy();
        let mut name = String::with_capacity(1 + file.len() + 1);

        name.push('@');
        name.push_str(&file);
        name.push('\0');

        // SAFETY: We use CStr::from_ptr() instead of CStr::from_bytes_with_nul_unchecked() since
        // file path may contains interior NUL.
        let name = unsafe { CStr::from_ptr(name.as_ptr().cast()) };

        Ok(self.load(Some(name), ty, data))
    }

    #[inline(always)]
    fn push_nil(&mut self) -> Nil<Self> {
        unsafe { zl_pushnil(self.state()) };
        unsafe { Nil::new(self) }
    }

    #[inline(always)]
    fn push_bool(&mut self, v: bool) -> Bool<Self> {
        unsafe { zl_pushboolean(self.state(), v) };
        unsafe { Bool::new(self) }
    }

    #[inline(always)]
    fn push_str(&mut self, v: impl AsRef<[u8]>) -> Str<Self> {
        let v = v.as_ref();

        unsafe { zl_pushlstring(self.state(), v.as_ptr().cast(), v.len()) };
        unsafe { Str::new(self) }
    }

    #[inline(always)]
    fn push_table(&mut self, narr: u16, nrec: u16) -> Table<Self> {
        unsafe { zl_createtable(self.state(), narr.into(), nrec.into()) };

        unsafe { Table::new(self) }
    }

    fn push_iter<T, I>(&mut self, v: T) -> Iter<Self>
    where
        T: IntoIterator<Item: IntoLua, IntoIter = I>,
        I: Iterator<Item = T::Item> + 'static,
    {
        let v = v.into_iter().fuse();

        if align_of::<Fuse<I>>() > align_of::<*mut ()>() {
            // Push iterator function.
            unsafe { zl_pushcclosure(self.state(), self::iter::next::<Box<Fuse<I>>>, 0) };

            // Push state.
            let ptr = unsafe { zl_newuserdatauv(self.state(), size_of::<Box<Fuse<I>>>(), 0) };

            unsafe { ptr.cast::<Box<Fuse<I>>>().write(v.into()) };

            // Set finalizer.
            unsafe { zl_createtable(self.state(), 0, 1) };
            unsafe { zl_pushcclosure(self.state(), finalizer::<Box<Fuse<I>>>, 0) };
            unsafe { zl_setfield(self.state(), -2, c"__gc".as_ptr()) };
            unsafe { zl_setmetatable(self.state(), -2) };
        } else {
            // Push iterator function.
            unsafe { zl_pushcclosure(self.state(), self::iter::next::<Fuse<I>>, 0) };

            // Push state.
            let ptr = unsafe { zl_newuserdatauv(self.state(), size_of::<Fuse<I>>(), 0) };

            unsafe { ptr.cast::<Fuse<I>>().write(v) };

            // Set finalizer.
            if std::mem::needs_drop::<Fuse<I>>() {
                unsafe { zl_createtable(self.state(), 0, 1) };
                unsafe { zl_pushcclosure(self.state(), finalizer::<Fuse<I>>, 0) };
                unsafe { zl_setfield(self.state(), -2, c"__gc".as_ptr()) };
                unsafe { zl_setmetatable(self.state(), -2) };
            }
        }

        // Push control variable.
        unsafe { zl_pushnil(self.state()) };

        unsafe { Iter::new(self) }
    }

    /// # Panics
    /// If `T` was not registered with [`Frame::register_ud()`].
    fn push_ud<T: UserType>(&mut self, v: T) -> OwnedUd<Self, T> {
        // Create userdata.
        let nuvalue = T::user_values().map(|v| v.get()).unwrap_or(0).into();
        let ptr = if is_boxed::<T>() {
            let ptr = unsafe { zl_newuserdatauv(self.state(), size_of::<Box<T>>(), nuvalue) };
            let ptr = ptr.cast::<Box<T>>();

            unsafe { ptr.write(v.into()) };
            unsafe { (*ptr).as_ref() as *const T }
        } else {
            let ptr = unsafe { zl_newuserdatauv(self.state(), size_of::<T>(), nuvalue) };
            let ptr = ptr.cast::<T>();

            unsafe { ptr.write(v) };
            ptr
        };

        // Set metatable.
        unsafe { push_metatable::<T>(self.state()) };
        unsafe { zl_setmetatable(self.state(), -2) };

        unsafe { OwnedUd::new(self, ptr) }
    }

    /// See [`Context`] for how to return some values to Lua.
    fn push_fn<F>(&mut self, f: F) -> Function<Self>
    where
        F: Fn(&mut Context<NonYieldable>) -> Result<(), Error> + 'static,
    {
        if size_of::<F>() == 0 {
            unsafe { zl_pushcclosure(self.state(), invoker::<F>, 0) };
        } else if align_of::<F>() <= align_of::<*mut ()>() {
            // Move Rust function to Lua user data.
            let ptr = unsafe { zl_newuserdatauv(self.state(), size_of::<F>(), 0) };

            unsafe { ptr.cast::<F>().write(f) };

            // Set finalizer.
            if std::mem::needs_drop::<F>() {
                unsafe { zl_createtable(self.state(), 0, 1) };
                unsafe { zl_pushcclosure(self.state(), finalizer::<F>, 0) };
                unsafe { zl_setfield(self.state(), -2, c"__gc".as_ptr()) };
                unsafe { zl_setmetatable(self.state(), -2) };
            }

            // Push invoker.
            unsafe { zl_pushcclosure(self.state(), invoker::<F>, 1) };
        } else {
            // Move Rust function to Lua user data.
            let ptr = unsafe { zl_newuserdatauv(self.state(), size_of::<Box<F>>(), 0) };

            unsafe { ptr.cast::<Box<F>>().write(f.into()) };

            // Set finalizer.
            unsafe { zl_createtable(self.state(), 0, 1) };
            unsafe { zl_pushcclosure(self.state(), finalizer::<Box<F>>, 0) };
            unsafe { zl_setfield(self.state(), -2, c"__gc".as_ptr()) };
            unsafe { zl_setmetatable(self.state(), -2) };

            // Push invoker.
            unsafe { zl_pushcclosure(self.state(), invoker::<Box<F>>, 1) };
        }

        unsafe { Function::new(self) }
    }

    /// See [`Context`] for how to return some values to Lua.
    fn push_async<F>(&mut self, f: F) -> Function<Self>
    where
        F: AsyncFn(&mut Context<Yieldable>) -> Result<(), Error> + 'static,
    {
        if size_of::<F>() == 0 {
            unsafe { zl_pushcclosure(self.state(), async_invoker::<F>, 0) };
        } else if align_of::<F>() <= align_of::<*mut ()>() {
            // Move Rust function to Lua user data.
            let ptr = unsafe { zl_newuserdatauv(self.state(), size_of::<F>(), 0) };

            unsafe { ptr.cast::<F>().write(f) };

            // Set finalizer.
            if std::mem::needs_drop::<F>() {
                unsafe { zl_createtable(self.state(), 0, 1) };
                unsafe { zl_pushcclosure(self.state(), finalizer::<F>, 0) };
                unsafe { zl_setfield(self.state(), -2, c"__gc".as_ptr()) };
                unsafe { zl_setmetatable(self.state(), -2) };
            }

            // Push invoker.
            unsafe { zl_pushcclosure(self.state(), async_invoker::<F>, 1) };
        } else {
            // Move Rust function to Lua user data.
            let ptr = unsafe { zl_newuserdatauv(self.state(), size_of::<Box<F>>(), 0) };

            unsafe { ptr.cast::<Box<F>>().write(f.into()) };

            // Set finalizer.
            unsafe { zl_createtable(self.state(), 0, 1) };
            unsafe { zl_pushcclosure(self.state(), finalizer::<Box<F>>, 0) };
            unsafe { zl_setfield(self.state(), -2, c"__gc".as_ptr()) };
            unsafe { zl_setmetatable(self.state(), -2) };

            // Push invoker.
            unsafe { zl_pushcclosure(self.state(), async_invoker::<Box<F>>, 1) };
        }

        unsafe { Function::new(self) }
    }

    /// # Errors
    /// If the stack cannot grow to the requested size (e.g. maximum stack has been reached or out
    /// of memory).
    #[inline(always)]
    fn ensure_stack(&mut self, n: PositiveInt) {
        unsafe { zl_checkstack(self.state(), n.get()) };
    }
}

impl<T: RawState> Frame for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Lua;

    #[test]
    fn load_ok() {
        let mut lua = Lua::new(None).unwrap();

        lua.load(None, ChunkType::Text, "return 7").unwrap();
    }
}
