use std::ffi::c_int;
use std::num::NonZero;

/// Encapsulates a [`c_int`] greater than zero.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PositiveInt(NonZero<c_int>);

impl PositiveInt {
    pub const fn new(v: c_int) -> Option<Self> {
        let v = match NonZero::new(v) {
            Some(v) => v,
            None => return None,
        };

        if v.get() > 0 { Some(Self(v)) } else { None }
    }

    pub const fn get(self) -> c_int {
        self.0.get()
    }
}
