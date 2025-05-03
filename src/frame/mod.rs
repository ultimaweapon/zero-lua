pub use self::r#yield::*;

use self::r#async::async_invoker;
use self::function::invoker;
use self::userdata::{finalizer, push_metatable};
use crate::convert::IntoLua;
use crate::ffi::{
    ZL_REGISTRYINDEX, zl_checkstack, zl_createtable, zl_load, zl_newmetatable, zl_newuserdatauv,
    zl_pop, zl_pushboolean, zl_pushcclosure, zl_pushlstring, zl_pushnil, zl_require_base,
    zl_require_coroutine, zl_require_io, zl_require_math, zl_require_os, zl_require_string,
    zl_require_table, zl_require_utf8, zl_setfield, zl_setmetatable,
};
use crate::state::FrameState;
use crate::{
    Bool, ChunkType, Context, Error, Function, GlobalSetter, Iter, MainState, Nil, NonYieldable,
    PositiveInt, Str, Table, TableFrame, TableGetter, TableSetter, UserData, UserType, Value,
    Yieldable, is_boxed,
};
use std::any::TypeId;
use std::ffi::CStr;
use std::iter::Fuse;
use std::mem::ManuallyDrop;
use std::panic::RefUnwindSafe;
use std::path::Path;

mod r#async;
mod function;
mod iter;
mod userdata;
mod r#yield;

/// Virtual frame in a Lua stack.
///
/// Some methods in this trait can raise a Lua error. When calling outside Lua runtime it will
/// trigger Lua panic, which terminate the process.
pub trait Frame: FrameState {
    /// Returns `true` if `T` was successfully registered or `false` if the other userdata with the
    /// same name already registered.
    fn register_ud<T: UserType>(&mut self) -> bool
    where
        Self: FrameState<State = MainState>,
    {
        if unsafe { zl_newmetatable(self.state().get(), T::name().as_ptr()) == 0 } {
            unsafe { zl_pop(self.state().get(), 1) };
            return false;
        }

        T::setup_metatable(&mut ManuallyDrop::new(unsafe { Table::new(self) }));

        // Set "typeid".
        let ud = unsafe { zl_newuserdatauv(self.state().get(), size_of::<TypeId>(), 0) };

        unsafe { ud.cast::<TypeId>().write_unaligned(TypeId::of::<T>()) };
        unsafe { zl_setfield(self.state().get(), -2, c"typeid".as_ptr()) };

        // Set finalizer.
        if is_boxed::<T>() {
            unsafe { zl_pushcclosure(self.state().get(), finalizer::<Box<T>>, 0) };
            unsafe { zl_setfield(self.state().get(), -2, c"__gc".as_ptr()) };
        } else if std::mem::needs_drop::<T>() {
            unsafe { zl_pushcclosure(self.state().get(), finalizer::<T>, 0) };
            unsafe { zl_setfield(self.state().get(), -2, c"__gc".as_ptr()) };
        }

        unsafe { zl_pop(self.state().get(), 1) };

        // Add to global.
        T::setup_global(GlobalSetter::new(self, T::name()));

        true
    }

    #[inline(always)]
    fn require_base(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_base(self.state().get(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_coroutine(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_coroutine(self.state().get(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_io(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_io(self.state().get(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_math(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_math(self.state().get(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_os(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_os(self.state().get(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_string(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_string(self.state().get(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_table(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_table(self.state().get(), global) };
        unsafe { Table::new(self) }
    }

    #[inline(always)]
    fn require_utf8(&mut self, global: bool) -> Table<Self> {
        unsafe { zl_require_utf8(self.state().get(), global) };
        unsafe { Table::new(self) }
    }

    fn set_registry<K: TableSetter>(&mut self, k: K) -> TableFrame<Self, K> {
        unsafe { TableFrame::new(self, ZL_REGISTRYINDEX, k) }
    }

    fn get_registry<K: TableGetter>(&mut self, k: K) -> Value<Self> {
        unsafe { Value::from_table(self, ZL_REGISTRYINDEX, k) }
    }

    fn set_global<N: AsRef<CStr>>(&mut self, name: N) -> GlobalSetter<Self, N> {
        GlobalSetter::new(self, name)
    }

    fn load(
        &mut self,
        name: impl AsRef<CStr>,
        ty: ChunkType,
        chunk: impl AsRef<[u8]>,
    ) -> Result<Function<Self>, Str<Self>> {
        let name = name.as_ref();
        let chunk = chunk.as_ref();
        let mode = ty.to_c_str();

        match unsafe {
            zl_load(
                self.state().get(),
                name.as_ptr(),
                chunk.as_ptr().cast(),
                chunk.len(),
                mode.as_ptr(),
            )
        } {
            true => Ok(unsafe { Function::new(self) }),
            false => Err(unsafe { Str::new(self) }),
        }
    }

    /// This method will load the whole content of `file` into memory before passing to Lua.
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

        // Load.
        let name = name.as_ptr().cast();
        let mode = ty.to_c_str();
        let r = match unsafe {
            zl_load(
                self.state().get(),
                name,
                data.as_ptr().cast(),
                data.len(),
                mode.as_ptr(),
            )
        } {
            true => Ok(unsafe { Function::new(self) }),
            false => Err(unsafe { Str::new(self) }),
        };

        Ok(r)
    }

    #[inline(always)]
    fn push_nil(&mut self) -> Nil<Self> {
        unsafe { zl_pushnil(self.state().get()) };
        unsafe { Nil::new(self) }
    }

    #[inline(always)]
    fn push_bool(&mut self, v: bool) -> Bool<Self> {
        unsafe { zl_pushboolean(self.state().get(), v) };
        unsafe { Bool::new(self) }
    }

    #[inline(always)]
    fn push_str(&mut self, v: impl AsRef<[u8]>) -> Str<Self> {
        let v = v.as_ref();

        unsafe { zl_pushlstring(self.state().get(), v.as_ptr().cast(), v.len()) };
        unsafe { Str::new(self) }
    }

    #[inline(always)]
    fn push_table(&mut self, narr: u16, nrec: u16) -> Table<Self> {
        unsafe { zl_createtable(self.state().get(), narr.into(), nrec.into()) };

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
            unsafe { zl_pushcclosure(self.state().get(), self::iter::next::<Box<Fuse<I>>>, 0) };

            // Push state.
            let ptr = unsafe { zl_newuserdatauv(self.state().get(), size_of::<Box<Fuse<I>>>(), 0) };

            unsafe { ptr.cast::<Box<Fuse<I>>>().write(v.into()) };

            // Set finalizer.
            unsafe { zl_createtable(self.state().get(), 0, 1) };
            unsafe { zl_pushcclosure(self.state().get(), finalizer::<Box<Fuse<I>>>, 0) };
            unsafe { zl_setfield(self.state().get(), -2, c"__gc".as_ptr()) };
            unsafe { zl_setmetatable(self.state().get(), -2) };
        } else {
            // Push iterator function.
            unsafe { zl_pushcclosure(self.state().get(), self::iter::next::<Fuse<I>>, 0) };

            // Push state.
            let ptr = unsafe { zl_newuserdatauv(self.state().get(), size_of::<Fuse<I>>(), 0) };

            unsafe { ptr.cast::<Fuse<I>>().write(v) };

            // Set finalizer.
            if std::mem::needs_drop::<Fuse<I>>() {
                unsafe { zl_createtable(self.state().get(), 0, 1) };
                unsafe { zl_pushcclosure(self.state().get(), finalizer::<Fuse<I>>, 0) };
                unsafe { zl_setfield(self.state().get(), -2, c"__gc".as_ptr()) };
                unsafe { zl_setmetatable(self.state().get(), -2) };
            }
        }

        // Push control variable.
        unsafe { zl_pushnil(self.state().get()) };

        unsafe { Iter::new(self) }
    }

    /// # Panics
    /// If `T` was not registered with [`Frame::register_ud()`].
    fn push_ud<T: UserType>(&mut self, v: T) -> UserData<Self> {
        // Create userdata.
        let nuvalue = T::user_values().map(|v| v.get()).unwrap_or(0).into();

        if is_boxed::<T>() {
            let ptr = unsafe { zl_newuserdatauv(self.state().get(), size_of::<Box<T>>(), nuvalue) };

            unsafe { ptr.cast::<Box<T>>().write(v.into()) };
        } else {
            let ptr = unsafe { zl_newuserdatauv(self.state().get(), size_of::<T>(), nuvalue) };

            unsafe { ptr.cast::<T>().write(v) };
        }

        // Set metatable.
        unsafe { push_metatable::<T>(self.state().get()) };
        unsafe { zl_setmetatable(self.state().get(), -2) };

        unsafe { UserData::new(self) }
    }

    /// See [`Context`] for how to return some values to Lua.
    fn push_fn<F>(&mut self, f: F) -> Function<Self>
    where
        F: Fn(&mut Context<NonYieldable>) -> Result<(), Error> + RefUnwindSafe + 'static,
    {
        if size_of::<F>() == 0 {
            unsafe { zl_pushcclosure(self.state().get(), invoker::<F>, 0) };
        } else if align_of::<F>() <= align_of::<*mut ()>() {
            // Move Rust function to Lua user data.
            let ptr = unsafe { zl_newuserdatauv(self.state().get(), size_of::<F>(), 0) };

            unsafe { ptr.cast::<F>().write(f) };

            // Set finalizer.
            if std::mem::needs_drop::<F>() {
                unsafe { zl_createtable(self.state().get(), 0, 1) };
                unsafe { zl_pushcclosure(self.state().get(), finalizer::<F>, 0) };
                unsafe { zl_setfield(self.state().get(), -2, c"__gc".as_ptr()) };
                unsafe { zl_setmetatable(self.state().get(), -2) };
            }

            // Push invoker.
            unsafe { zl_pushcclosure(self.state().get(), invoker::<F>, 1) };
        } else {
            // Move Rust function to Lua user data.
            let ptr = unsafe { zl_newuserdatauv(self.state().get(), size_of::<Box<F>>(), 0) };

            unsafe { ptr.cast::<Box<F>>().write(f.into()) };

            // Set finalizer.
            unsafe { zl_createtable(self.state().get(), 0, 1) };
            unsafe { zl_pushcclosure(self.state().get(), finalizer::<Box<F>>, 0) };
            unsafe { zl_setfield(self.state().get(), -2, c"__gc".as_ptr()) };
            unsafe { zl_setmetatable(self.state().get(), -2) };

            // Push invoker.
            unsafe { zl_pushcclosure(self.state().get(), invoker::<Box<F>>, 1) };
        }

        unsafe { Function::new(self) }
    }

    /// See [`Context`] for how to return some values to Lua.
    fn push_async<F>(&mut self, f: F) -> Function<Self>
    where
        F: AsyncFn(&mut Context<Yieldable>) -> Result<(), Error> + RefUnwindSafe + 'static,
    {
        if size_of::<F>() == 0 {
            unsafe { zl_pushcclosure(self.state().get(), async_invoker::<F>, 0) };
        } else if align_of::<F>() <= align_of::<*mut ()>() {
            // Move Rust function to Lua user data.
            let ptr = unsafe { zl_newuserdatauv(self.state().get(), size_of::<F>(), 0) };

            unsafe { ptr.cast::<F>().write(f) };

            // Set finalizer.
            if std::mem::needs_drop::<F>() {
                unsafe { zl_createtable(self.state().get(), 0, 1) };
                unsafe { zl_pushcclosure(self.state().get(), finalizer::<F>, 0) };
                unsafe { zl_setfield(self.state().get(), -2, c"__gc".as_ptr()) };
                unsafe { zl_setmetatable(self.state().get(), -2) };
            }

            // Push invoker.
            unsafe { zl_pushcclosure(self.state().get(), async_invoker::<F>, 1) };
        } else {
            // Move Rust function to Lua user data.
            let ptr = unsafe { zl_newuserdatauv(self.state().get(), size_of::<Box<F>>(), 0) };

            unsafe { ptr.cast::<Box<F>>().write(f.into()) };

            // Set finalizer.
            unsafe { zl_createtable(self.state().get(), 0, 1) };
            unsafe { zl_pushcclosure(self.state().get(), finalizer::<Box<F>>, 0) };
            unsafe { zl_setfield(self.state().get(), -2, c"__gc".as_ptr()) };
            unsafe { zl_setmetatable(self.state().get(), -2) };

            // Push invoker.
            unsafe { zl_pushcclosure(self.state().get(), async_invoker::<Box<F>>, 1) };
        }

        unsafe { Function::new(self) }
    }

    /// # Error
    /// If the stack cannot grow to the requested size.
    #[inline(always)]
    fn ensure_stack(&mut self, n: PositiveInt) {
        unsafe { zl_checkstack(self.state().get(), n.get()) };
    }

    #[inline(always)]
    fn as_yield(&mut self) -> Yield<Self>
    where
        Self: FrameState<State = Yieldable>,
    {
        Yield::new(self)
    }
}

impl<T: FrameState> Frame for T {}
