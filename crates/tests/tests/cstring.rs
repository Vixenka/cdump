use std::ffi::{c_char, CStr};

use cdump::{CDebug, CDeserialize, CSerialize};
use tests::eval_debug;

#[derive(CDebug, CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    a: u32,
    text: *const c_char,
}

#[test]
fn cstring() {
    let text = c"Hello world!";
    let obj = Foo {
        a: 1984,
        text: text.as_ptr(),
    };

    eval_debug(&obj);

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { Foo::deserialize(&mut reader) };

    eval_debug(&copy);
    assert_eq!(obj.a, copy.a);
    assert_ne!(obj.text, copy.text);
    unsafe {
        assert_eq!(CStr::from_ptr(obj.text), CStr::from_ptr(copy.text));
    }
}
