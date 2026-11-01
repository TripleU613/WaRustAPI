//! Render local decoder diagnostics; no Android logging service is emulated
//! (liblog.so). Rust port of the former native-log-imports.c.
#![no_std]
#![allow(clippy::missing_safety_doc)]

use core::ffi::{VaList, c_char, c_int};

unsafe extern "C" {
    fn puts(s: *const c_char) -> c_int;
    fn vprintf(format: *const c_char, args: VaList) -> c_int;
    fn abort() -> !;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { abort() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __android_log_print(
    _priority: c_int,
    tag: *const c_char,
    format: *const c_char,
    args: ...
) -> c_int {
    unsafe {
        puts(tag);
        let n = vprintf(format, args);
        puts(c"".as_ptr());
        n
    }
}
