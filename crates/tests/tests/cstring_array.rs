use std::ffi::{c_char, CStr};

use cdump::{CDeserialize, CSerialize};

#[derive(Debug, CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    len: u32,
    #[cdump(array(len = len))]
    text: *const *const c_char,
}

#[test]
fn cstring() {
    let text1 = c"Hello world!";
    let text2 = c"Hello miyazaki!";
    let array = [text1.as_ptr(), text2.as_ptr()];
    let obj = Foo {
        len: array.len() as u32,
        text: array.as_ptr(),
    };

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { Foo::deserialize(&mut reader) };

    assert_eq!(obj.len, copy.len);
    assert_ne!(obj.text, copy.text);
    unsafe {
        assert_ne!(*obj.text, *copy.text);
        assert_eq!(CStr::from_ptr(*obj.text), CStr::from_ptr(*copy.text));
        assert_ne!(*obj.text.add(1), *copy.text.add(1));
        assert_eq!(
            CStr::from_ptr(*obj.text.add(1)),
            CStr::from_ptr(*copy.text.add(1))
        );
    }
}
