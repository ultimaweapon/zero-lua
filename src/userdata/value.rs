use super::{OwnedUd, UserType, is_boxed};
use crate::ffi::{zl_getfield, zl_getmetatable, zl_pop, zl_touserdata};
use crate::state::FrameState;
use crate::{Frame, TYPE_ID, Unknown};
use std::any::TypeId;
use std::ffi::c_int;
use std::mem::ManuallyDrop;
use std::ops::DerefMut;

/// Represents a full userdata on the top of stack.
pub struct UserData<'p, P: Frame>(&'p mut P);

impl<'p, P: Frame> UserData<'p, P> {
    /// # Safety
    /// Top of the stack must be a full userdata.
    #[inline(always)]
    pub(crate) unsafe fn new(p: *mut P) -> Self {
        Self(unsafe { &mut *p })
    }

    pub fn downcast<T: UserType>(mut self) -> Result<OwnedUd<'p, P, T>, Self> {
        // Get metatable.
        if unsafe { zl_getmetatable(self.state().get(), -1) == 0 } {
            return Err(self);
        }

        unsafe { zl_getfield(self.state().get(), -1, TYPE_ID.as_ptr()) };

        // SAFETY: TypeId is Copy.
        let id = TypeId::of::<T>();
        let ud = unsafe { zl_touserdata(self.state().get(), -1) };
        let ok = unsafe { !ud.is_null() && ud.cast::<TypeId>().read_unaligned() == id };

        unsafe { zl_pop(self.state().get(), 2) };

        if !ok {
            return Err(self);
        }

        // Get pointer to UD.
        let ptr = unsafe { zl_touserdata(self.state().get(), -1).cast_const() };
        let ptr = if is_boxed::<T>() {
            unsafe { (*ptr.cast::<Box<T>>()).as_ref() }
        } else {
            ptr.cast::<T>()
        };

        Ok(unsafe { OwnedUd::new(ManuallyDrop::new(self).deref_mut().0, ptr) })
    }

    #[inline(always)]
    pub fn into_unknown(self) -> Unknown<'p, P> {
        unsafe { Unknown::new(ManuallyDrop::new(self).deref_mut().0) }
    }
}

impl<P: Frame> Drop for UserData<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}

impl<'p, P: Frame, T> From<OwnedUd<'p, P, T>> for UserData<'p, P> {
    #[inline(always)]
    fn from(value: OwnedUd<'p, P, T>) -> Self {
        value.into_ud()
    }
}

impl<P: Frame> FrameState for UserData<'_, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.0.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state().get(), n) };
    }
}

impl<'p, P: Frame> From<UserData<'p, P>> for Unknown<'p, P> {
    #[inline(always)]
    fn from(value: UserData<'p, P>) -> Self {
        value.into_unknown()
    }
}
