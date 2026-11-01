//! Local ARM64 artifact probe, not a Java VM or WhatsApp client.
//! JNI table indexes: Oracle JNI Functions/Invocation API; ARM64 registrations
//! and range signature: APK 2.26.37.71 libsuperpack.so, 0x96d0 and 0x9a50.
//! All unsupported callbacks abort. Strings are ASCII fixture paths only.
//! Rust port of the former native-probe.c; build: README.md in this directory.
#![no_std]
#![no_main]
#![allow(static_mut_refs, clippy::missing_safety_doc)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr::null_mut;

unsafe extern "C" {
    fn __libc_init(
        raw: *mut c_void,
        onexit: *mut c_void,
        main: unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_char) -> c_int,
        structors: *const c_void,
    ) -> !;
    fn puts(s: *const c_char) -> c_int;
    fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn printf(format: *const c_char, ...) -> c_int;
    fn abort() -> !;
    fn calloc(n: usize, size: usize) -> *mut c_void;
    fn strdup(s: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { abort() }
}

#[repr(C)]
struct NativeMethod {
    name: *const c_char,
    sig: *const c_char,
    fn_ptr: *mut c_void,
}

#[repr(C)]
struct StringArray {
    length: c_int,
    items: *mut *mut c_char,
}

static mut ENV_TABLE: [*mut c_void; 234] = [null_mut(); 234];
static mut JNI_ENV: *mut c_void = null_mut();
static mut VM_TABLE: [*mut c_void; 8] = [null_mut(); 8];
static mut JNI_VM: *mut c_void = null_mut();
static mut REGISTERED: c_int = 0;
static mut EXCEPTION_PENDING: bool = false;
static mut RANGE_ENTRY: *mut c_void = null_mut();
static mut FULL_ENTRY: *mut c_void = null_mut();

unsafe extern "C" fn unsupported() -> ! {
    unsafe {
        puts(c"unsupported JNI callback".as_ptr());
        abort()
    }
}

unsafe fn eq(a: *const c_char, b: &CStr) -> bool {
    unsafe { strcmp(a, b.as_ptr()) == 0 }
}

unsafe extern "C" fn find_class(_env: *mut c_void, name: *const c_char) -> *mut c_void {
    let known = [
        c"com/facebook/superpack/AssetDecompressor",
        c"com/whatsapp/superpack/WhatsAppObiInputStream",
        c"com/whatsapp/superpack/WhatsAppOpenboxArchive",
        c"java/lang/String",
        c"com/facebook/superpack/AssetDecompressionException",
    ];
    unsafe {
        if !known.iter().any(|k| eq(name, k)) {
            puts(name);
            unsupported();
        }
    }
    name as *mut c_void
}

unsafe extern "C" fn exception_check(_env: *mut c_void) -> u8 {
    unsafe { EXCEPTION_PENDING as u8 }
}

unsafe extern "C" fn throw_new(
    _env: *mut c_void,
    klass: *const c_char,
    message: *const c_char,
) -> c_int {
    unsafe {
        printf(
            c"native exception class=%s reason=%s\n".as_ptr(),
            klass,
            message,
        );
        EXCEPTION_PENDING = true;
    }
    0
}

unsafe fn ascii(s: *const c_char) {
    unsafe {
        if s.is_null() || strlen(s) > 4096 {
            unsupported();
        }
        if CStr::from_ptr(s).to_bytes().iter().any(|&b| b >= 128) {
            unsupported();
        }
    }
}

unsafe extern "C" fn get_utf(_env: *mut c_void, s: *mut c_char, copy: *mut u8) -> *mut c_char {
    unsafe {
        ascii(s);
        if !copy.is_null() {
            *copy = 0;
        }
    }
    s
}

unsafe extern "C" fn release_utf(_env: *mut c_void, s: *mut c_char, chars: *const c_char) {
    if s as *const c_char != chars {
        unsafe { unsupported() }
    }
}

unsafe extern "C" fn new_utf(_env: *mut c_void, s: *const c_char) -> *mut c_char {
    unsafe {
        ascii(s);
        let r = strdup(s);
        if r.is_null() {
            unsupported();
        }
        r
    }
}

/// Objects live until this isolated probe exits.
unsafe extern "C" fn delete_ref(_env: *mut c_void, _ref: *mut c_void) {}

unsafe extern "C" fn new_array(
    _env: *mut c_void,
    length: c_int,
    klass: *const c_char,
    init: *mut c_void,
) -> *mut StringArray {
    unsafe {
        if !(0..=1024).contains(&length) || !init.is_null() || !eq(klass, c"java/lang/String") {
            unsupported();
        }
        let a = calloc(1, size_of::<StringArray>()) as *mut StringArray;
        if a.is_null() {
            unsupported();
        }
        (*a).length = length;
        (*a).items = calloc(length.max(1) as usize, size_of::<*mut c_char>()) as *mut *mut c_char;
        if (*a).items.is_null() {
            unsupported();
        }
        a
    }
}

unsafe extern "C" fn set_array(
    _env: *mut c_void,
    array: *mut StringArray,
    index: c_int,
    value: *mut c_char,
) {
    unsafe {
        if array.is_null() || index < 0 || index >= (*array).length {
            unsupported();
        }
        ascii(value);
        *(*array).items.add(index as usize) = value;
    }
}

unsafe extern "C" fn register_natives(
    _env: *mut c_void,
    klass: *const c_char,
    methods: *const NativeMethod,
    count: c_int,
) -> c_int {
    unsafe {
        printf(c"register class=%s count=%d\n".as_ptr(), klass, count);
        if !(0..=32).contains(&count) {
            unsupported();
        }
        let methods = core::slice::from_raw_parts(methods, count as usize);
        for m in methods {
            if m.name.is_null() || m.sig.is_null() || m.fn_ptr.is_null() {
                unsupported();
            }
            printf(c"native=%s signature=%s\n".as_ptr(), m.name, m.sig);
        }
        for m in methods {
            if eq(m.name, c"decompress_range_from_so") {
                if !eq(m.sig, c"(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;II)[Ljava/lang/String;") || !RANGE_ENTRY.is_null() {
                    unsupported();
                }
                RANGE_ENTRY = m.fn_ptr;
            }
        }
        for m in methods {
            if eq(m.name, c"decompress_from_so") {
                if !eq(
                    m.sig,
                    c"(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)[Ljava/lang/String;",
                ) || !FULL_ENTRY.is_null()
                {
                    unsupported();
                }
                FULL_ENTRY = m.fn_ptr;
            }
        }
        REGISTERED += count;
    }
    0
}

unsafe extern "C" fn get_env(_vm: *mut c_void, out: *mut *mut c_void, version: c_int) -> c_int {
    if version != 0x10006 {
        return -3;
    }
    unsafe { *out = &raw mut JNI_ENV as *mut c_void };
    0
}

type FullEntry = unsafe extern "C" fn(
    *mut c_void,
    *mut c_void,
    *const c_char,
    *const c_char,
    *const c_char,
) -> *mut StringArray;
type RangeEntry = unsafe extern "C" fn(
    *mut c_void,
    *mut c_void,
    *const c_char,
    *const c_char,
    *const c_char,
    c_int,
    c_int,
) -> *mut StringArray;

unsafe extern "C" fn main(argc: c_int, argv: *mut *mut c_char, _envp: *mut *mut c_char) -> c_int {
    unsafe {
        let decoder = dlopen(c"/system/lib64/libsuperpack.so".as_ptr(), 2);
        if decoder.is_null() {
            puts(dlerror());
            return 2;
        }
        let entry = dlsym(decoder, c"JNI_OnLoad".as_ptr());
        if entry.is_null() {
            puts(dlerror());
            return 3;
        }
        puts(c"decoder loaded; JNI_OnLoad resolved".as_ptr());
        for slot in &mut ENV_TABLE[4..] {
            *slot = unsupported as *mut c_void;
        }
        for slot in &mut VM_TABLE[3..] {
            *slot = unsupported as *mut c_void;
        }
        // JNI function table slots, not APK protobuf fields.
        ENV_TABLE[6] = find_class as *mut c_void;
        ENV_TABLE[14] = throw_new as *mut c_void;
        ENV_TABLE[23] = delete_ref as *mut c_void;
        ENV_TABLE[167] = new_utf as *mut c_void;
        ENV_TABLE[169] = get_utf as *mut c_void;
        ENV_TABLE[170] = release_utf as *mut c_void;
        ENV_TABLE[172] = new_array as *mut c_void;
        ENV_TABLE[174] = set_array as *mut c_void;
        ENV_TABLE[215] = register_natives as *mut c_void;
        ENV_TABLE[228] = exception_check as *mut c_void;
        VM_TABLE[6] = get_env as *mut c_void;
        JNI_ENV = &raw mut ENV_TABLE as *mut c_void;
        JNI_VM = &raw mut VM_TABLE as *mut c_void;
        let on_load: unsafe extern "C" fn(*mut c_void, *mut c_void) -> c_int =
            core::mem::transmute(entry);
        let result = on_load(&raw mut JNI_VM as *mut c_void, null_mut());
        printf(
            c"JNI_OnLoad result=0x%x registered=%d\n".as_ptr(),
            result,
            REGISTERED,
        );
        if result != 0x10006 || REGISTERED != 19 || EXCEPTION_PENDING {
            return 4;
        }
        let arg = if argc == 2 { *argv.add(1) } else { null_mut() };
        if argc == 2 && (eq(arg, c"--first-entry") || eq(arg, c"--full-archive")) {
            // APK entry requests flags=6 (NOW|NOLOAD); preload this local ELF.
            let archive = dlopen(c"/system/lib64/wa_archive.so".as_ptr(), 2);
            if archive.is_null() {
                puts(dlerror());
                return 5;
            }
            if RANGE_ENTRY.is_null() {
                return 6;
            }
            let env = &raw mut JNI_ENV as *mut c_void;
            let (path, spo, out) = (
                c"/system/lib64/wa_archive.so".as_ptr(),
                c"spo".as_ptr(),
                c"/decoded".as_ptr(),
            );
            let files = if eq(arg, c"--full-archive") {
                if FULL_ENTRY.is_null() {
                    return 6;
                }
                let full: FullEntry = core::mem::transmute(FULL_ENTRY);
                full(env, null_mut(), path, spo, out)
            } else {
                // 0xa964 loops while index < end: [0,1) selects one output entry.
                let range: RangeEntry = core::mem::transmute(RANGE_ENTRY);
                range(env, null_mut(), path, spo, out, 0, 1)
            };
            if EXCEPTION_PENDING || files.is_null() {
                return 7;
            }
            printf(c"returned files=%d\n".as_ptr(), (*files).length);
            for i in 0..(*files).length as usize {
                puts(*(*files).items.add(i));
            }
        } else if argc != 1 {
            return 8;
        }
    }
    0
}

// Bionic's static-executable entry: __libc_init(raw_args, onexit, main,
// structors) with empty preinit/init/fini arrays ({-1, 0} terminated).
#[repr(C)]
struct Structors([*const c_void; 3]);
static CTORS: [usize; 2] = [usize::MAX, 0];

#[unsafe(no_mangle)]
unsafe extern "C" fn start_rust(raw: *mut c_void) -> ! {
    let ctors = &raw const CTORS as *const c_void;
    let structors = Structors([ctors, ctors, ctors]);
    unsafe { __libc_init(raw, null_mut(), main, &raw const structors as *const c_void) }
}

core::arch::global_asm!(".global _start", "_start:", "mov x0, sp", "b start_rust");
