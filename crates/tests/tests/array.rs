use std::{
    ffi::{c_char, CStr},
    fmt::Debug,
};

use cdump::{CDeserialize, CSerialize};

#[derive(Debug, CSerialize, CDeserialize)]
#[repr(C)]
struct DeepFoo {
    len: u32,
    #[cdump(array(len = len))]
    b: *const ShallowBar,
    c: f64,
}

#[derive(Copy, Clone, CSerialize, CDeserialize)]
#[repr(C)]
struct ShallowBar {
    a: f64,
    b: *const c_char,
    c: u32,
}

impl PartialEq for ShallowBar {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a
            && self.c == other.c
            && unsafe { CStr::from_ptr(self.b) == CStr::from_ptr(other.b) }
    }
}

impl Debug for ShallowBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = unsafe { CStr::from_ptr(self.b) };
        f.debug_struct("ShallowBar")
            .field("a", &self.a)
            .field("b", &text)
            .field("c", &self.c)
            .finish()
    }
}

#[test]
fn array() {
    let text1 = c"what";
    let text2 = c"11";

    let array = [
        ShallowBar {
            a: 19.84,
            b: text1.as_ptr(),
            c: 1864,
        },
        ShallowBar {
            a: 20.77,
            b: text2.as_ptr(),
            c: 7864,
        },
    ];
    let obj = DeepFoo {
        len: array.len() as u32,
        b: array.as_ptr(),
        c: 2024.07,
    };

    let mut buf = cdump::CDumpBufferWriter::new();
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { DeepFoo::deserialize(&mut reader) };

    assert_eq!(obj.len, copy.len);
    assert_ne!(obj.b, copy.b);
    assert_eq!(unsafe { *obj.b }, unsafe { *copy.b });
    assert_eq!(unsafe { *obj.b.add(1) }, unsafe { *copy.b.add(1) });
}
