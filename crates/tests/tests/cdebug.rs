use std::ffi::c_char;

use cdump::CDebug;
use tests::eval_debug;

#[derive(CDebug)]
struct NullCString {
    ptr: *const c_char,
}

#[test]
fn null_cstring() {
    let obj = NullCString {
        ptr: std::ptr::null(),
    };
    eval_debug(&obj);
}
