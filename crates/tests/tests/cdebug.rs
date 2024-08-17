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

#[derive(CDebug)]
struct CStringInlined {
    data: [c_char; 100],
}

#[test]
fn cstring_inlined() {
    let mut obj = CStringInlined { data: [0; 100] };
    let text = c"Kiss Me Again!";
    for (i, &c) in text.to_bytes().iter().enumerate() {
        obj.data[i] = c as c_char;
    }
    eval_debug(&obj);
}
