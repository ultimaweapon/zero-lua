/// Data associated with all `lua_State`.
pub struct ExtraData {
    pub panic: Box<dyn Fn(Option<&str>)>,
}
