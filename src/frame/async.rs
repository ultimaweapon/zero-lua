use crate::ffi::{
    lua_State, zl_error, zl_getextraspace, zl_pushlightuserdata, zl_touserdata, zl_upvalueindex,
    zl_yieldk,
};
use crate::{AsyncContext, Context, Error, Yieldable};
use std::ffi::c_int;
use std::panic::RefUnwindSafe;
use std::pin::Pin;
use std::ptr::null_mut;
use std::task::Poll;

pub unsafe extern "C-unwind" fn async_invoker<F>(
    #[allow(non_snake_case)] L: *mut lua_State,
) -> c_int
where
    F: AsyncFn(&mut Context<Yieldable>) -> Result<(), Error> + RefUnwindSafe + 'static,
{
    // Check if calling from Future::poll().
    let cx = unsafe { zl_getextraspace(L).cast::<*mut AsyncContext>() };
    let cx = unsafe { cx.replace(null_mut()) }; // SAFETY: Prevent downstream to access this.

    if cx.is_null() {
        let m = c"attempt to call async function from non-async block";
        unsafe { zl_error(L, m.as_ptr()) };
    }

    // Get closure.
    let cb = if size_of::<F>() == 0 {
        std::ptr::dangling::<F>()
    } else {
        let cb = unsafe { zl_upvalueindex(1) };
        let cb = unsafe { zl_touserdata(L, cb).cast::<F>() };

        cb.cast_const()
    };

    // SAFETY: All values in the Lua stack will not removed when we yield.
    let cx = unsafe { &mut *cx };
    let s = unsafe { Yieldable::new(L, cx.values.clone()) };
    let mut f = Box::pin(async move {
        let mut cx = Context::new(s);

        match unsafe { (*cb)(&mut cx).await } {
            Ok(_) => cx.into_results(),
            Err(e) => cx.raise(e),
        }
    });

    match f.as_mut().poll(cx.cx) {
        Poll::Ready(v) => {
            unsafe { zl_getextraspace(L).cast::<*mut AsyncContext>().write(cx) };
            v
        }
        Poll::Pending => unsafe { async_yield(L, f, cx) },
    }
}

unsafe fn async_yield<F>(state: *mut lua_State, f: Pin<Box<F>>, cx: &mut AsyncContext) -> !
where
    F: Future<Output = c_int>,
{
    // All lua_pushlightuserdata never fails.
    let f = unsafe { Pin::into_inner_unchecked(f) };
    let f = Box::into_raw(f);

    unsafe { zl_pushlightuserdata(state, drop::<F> as _) };
    unsafe { zl_pushlightuserdata(state, f.cast()) };
    unsafe { zl_pushlightuserdata(state, (cx as *mut AsyncContext).cast()) };

    unsafe { zl_yieldk(state, 3, f as isize, poll::<F>) };
}

unsafe fn drop<F>(f: *mut ()) {
    // SAFETY: We did not move out the value from Box.
    unsafe { std::mem::drop(Box::from_raw(f.cast::<F>())) };
}

unsafe extern "C-unwind" fn poll<F>(
    #[allow(non_snake_case)] L: *mut lua_State,
    _: c_int,
    ctx: isize,
) -> c_int
where
    F: Future<Output = c_int>,
{
    // Restore future.
    let f = unsafe { Box::from_raw(ctx as *mut F) };
    let mut f = Box::into_pin(f);

    // Poll.
    let cx = unsafe { zl_getextraspace(L).cast::<*mut AsyncContext>() };
    let cx = unsafe { cx.replace(null_mut()) }; // SAFETY: Prevent downstream to access this.
    let cx = unsafe { &mut *cx };

    match f.as_mut().poll(cx.cx) {
        Poll::Ready(v) => {
            unsafe { zl_getextraspace(L).cast::<*mut AsyncContext>().write(cx) };
            v
        }
        Poll::Pending => unsafe { async_yield(L, f, cx) },
    }
}
