use super::{TypedUd, UserData, UserFrame, UserType};
use crate::ffi::zl_pop;
use crate::state::FrameState;
use crate::{Frame, Unknown, Value};
use std::ffi::c_int;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::num::NonZero;
use std::ops::DerefMut;

/// Represents a full userdata on the top of stack.
pub struct OwnedUd<'p, P: Frame, T> {
    parent: &'p mut P,
    phantom: PhantomData<T>,
}

impl<'p, P: Frame, T> OwnedUd<'p, P, T> {
    #[inline(always)]
    pub(crate) unsafe fn new(parent: *mut P, _: *const T) -> Self {
        Self {
            parent: unsafe { &mut *parent },
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_ud(self) -> UserData<'p, P> {
        unsafe { UserData::new(ManuallyDrop::new(self).deref_mut().parent) }
    }

    #[inline(always)]
    pub fn into_unknown(self) -> Unknown<'p, P> {
        unsafe { Unknown::new(ManuallyDrop::new(self).deref_mut().parent) }
    }
}

impl<'p, P: Frame, T> Drop for OwnedUd<'p, P, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.parent.release_values(1) };
    }
}

impl<'p, P, T> TypedUd for OwnedUd<'p, P, T>
where
    P: Frame,
    T: UserType,
{
    type Type = T;

    #[inline(always)]
    fn set_uv(&mut self, n: NonZero<u16>) -> Option<UserFrame<Self>> {
        if T::user_values().map(move |v| n <= v).unwrap_or(false) {
            Some(unsafe { UserFrame::new(self, -2, n) })
        } else {
            None
        }
    }

    #[inline(always)]
    fn get_uv(&mut self, n: NonZero<u16>) -> Option<Value<Self>> {
        unsafe { Value::from_uv(self, -1, n.get()) }
    }
}

impl<'p, P: Frame, T> FrameState for OwnedUd<'p, P, T> {
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state().get(), n) };
    }
}

impl<'p, P: Frame, T> From<OwnedUd<'p, P, T>> for Unknown<'p, P> {
    #[inline(always)]
    fn from(value: OwnedUd<'p, P, T>) -> Self {
        value.into_unknown()
    }
}
