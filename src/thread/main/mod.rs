pub(crate) use self::state::*;

use super::AsyncLua;
use crate::ffi::{lua_State, zl_atpanic, zl_getextraspace, zl_pop, zl_tolstring, zl_type};
use crate::state::{ExtraData, FrameState};
use crate::{PanicHandler, Type};
use std::backtrace::Backtrace;
use std::ffi::c_int;
use std::pin::Pin;
use std::rc::Rc;

mod state;

/// Encapsulates a `lua_State` created by `lua_newstate`.
pub struct Lua(MainState);

impl Lua {
    /// Create a new `lua_State` using `luaL_newstate`. Returns [`None`] if `luaL_newstate` return
    /// null.
    ///
    /// Specify [`Some`] for `panic` if you want a custom handler for Lua panic otherwise Zero Lua
    /// will provides a default one that print the panic to stderr.
    ///
    /// You may want to change Lua warning function after this if your application is a GUI
    /// application.
    #[inline(always)]
    pub fn new(panic: Option<Box<PanicHandler>>) -> Option<Self> {
        // Get panic handler.
        let panic = panic.unwrap_or_else(|| {
            Box::new(|msg| {
                let bt = Backtrace::force_capture();

                match msg {
                    Some(msg) => eprintln!("Lua panicked ({msg}).\n\n{bt}"),
                    None => eprintln!("Lua panicked with an unknown error.\n\n{bt}"),
                }
            })
        });

        // Initialize lua_State.
        let state = MainState::new(panic)?;

        unsafe { zl_atpanic(state.get(), Some(Self::panic)) };

        Some(Self(state))
    }

    pub fn into_async(self) -> Pin<Rc<AsyncLua>> {
        AsyncLua::new(self.0)
    }

    extern "C" fn panic(state: *mut lua_State) -> c_int {
        // We can't let Lua trigger any error here so we need to check type of the error object.
        let msg = match unsafe { zl_type(state, -1) } {
            Type::String => unsafe {
                let mut len = 0;
                let ptr = zl_tolstring(state, -1, &mut len);
                let msg = std::slice::from_raw_parts(ptr.cast(), len);

                Some(String::from_utf8_lossy(msg))
            },
            _ => None,
        };

        // Invoke handler.
        let ex = unsafe { zl_getextraspace(state).cast::<*const ExtraData>().read() };

        unsafe { ((*ex).panic)(msg.as_ref().map(|v| v.as_ref())) };

        0
    }
}

impl FrameState for Lua {
    type State = MainState;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        &mut self.0
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.0.get(), n) };
    }
}
