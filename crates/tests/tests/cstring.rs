use std::ffi::{c_char, CStr};

use cdump::{CDeserialize, CSerialize};

#[derive(Debug, CSerialize, CDeserialize)]
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

    let mut buf = cdump::CDumpBufferWriter::new();
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { Foo::deserialize(&mut reader) };

    assert_eq!(obj.a, copy.a);
    assert_ne!(obj.text, copy.text);
    unsafe {
        assert_eq!(CStr::from_ptr(obj.text), CStr::from_ptr(copy.text));
    }
}
