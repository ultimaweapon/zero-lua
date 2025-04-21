use std::ffi::{CStr, c_char};

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn zl_panic(msg: *const c_char) -> ! {
    let msg = unsafe { CStr::from_ptr(msg) };

    panic!("{}", msg.to_str().unwrap());
}
