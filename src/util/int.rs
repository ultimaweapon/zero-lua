use std::cmp::Ordering;
use std::ffi::c_int;
use std::num::NonZero;

/// Encapsulates a [`c_int`] greater than zero.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PositiveInt(NonZero<c_int>);

impl PositiveInt {
    pub const ONE: Self = Self::new(1).unwrap();
    pub const TWO: Self = Self::new(2).unwrap();

    pub const fn new(v: c_int) -> Option<Self> {
        let v = match NonZero::new(v) {
            Some(v) => v,
            None => return None,
        };

        if v.get() > 0 { Some(Self(v)) } else { None }
    }

    /// # Safety
    /// `v` must be positive.
    pub const unsafe fn new_unchecked(v: c_int) -> Self {
        Self(unsafe { NonZero::new_unchecked(v) })
    }

    pub const fn get(self) -> c_int {
        self.0.get()
    }
}

impl PartialEq<c_int> for PositiveInt {
    #[inline(always)]
    fn eq(&self, other: &c_int) -> bool {
        self.get().eq(other)
    }
}

impl PartialOrd<c_int> for PositiveInt {
    #[inline(always)]
    fn partial_cmp(&self, other: &c_int) -> Option<Ordering> {
        self.get().partial_cmp(other)
    }
}
