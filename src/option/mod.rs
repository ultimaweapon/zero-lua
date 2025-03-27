/// Provides a function to convert Lua option string to Rust value.
pub trait FromOption: Sized {
    fn from_option(v: &[u8]) -> Option<Self>;
}

/// Represents an error when a Lua option is fails to parse.
pub struct OptionError(Vec<u8>);

impl OptionError {
    pub fn new(v: &[u8]) -> Self {
        // This mimic the same message as luaL_checkoption.
        let p = b"invalid option '";
        let mut m = Vec::with_capacity(p.len() + v.len() + 1);

        m.extend_from_slice(p);
        m.extend_from_slice(v);
        m.push(b'\'');

        Self(m)
    }
}

impl AsRef<[u8]> for OptionError {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
