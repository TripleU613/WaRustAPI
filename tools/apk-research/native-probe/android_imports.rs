//! Fail-closed asset imports for the local file-backed decoder probe
//! (libandroid.so). Rust port of the former native-android-imports.c.
#![no_std]
#![allow(clippy::missing_safety_doc)]

use core::ffi::{c_char, c_int, c_void};

unsafe extern "C" {
    fn puts(s: *const c_char) -> c_int;
    fn abort() -> !;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { abort() }
}

unsafe fn unsupported(name: &core::ffi::CStr) -> ! {
    unsafe {
        puts(name.as_ptr());
        abort()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn AAsset_close(_asset: *mut c_void) {
    unsafe { unsupported(c"unsupported asset import: AAsset_close") }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn AAssetManager_fromJava(
    _env: *mut c_void,
    _manager: *mut c_void,
) -> *mut c_void {
    unsafe { unsupported(c"unsupported asset import: AAssetManager_fromJava") }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn AAssetManager_open(
    _manager: *mut c_void,
    _name: *const c_char,
    _mode: c_int,
) -> *mut c_void {
    unsafe { unsupported(c"unsupported asset import: AAssetManager_open") }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn AAsset_openFileDescriptor(
    _asset: *mut c_void,
    _start: *mut c_void,
    _length: *mut c_void,
) -> c_int {
    unsafe { unsupported(c"unsupported asset import: AAsset_openFileDescriptor") }
}
