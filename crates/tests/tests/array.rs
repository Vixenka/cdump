use std::ffi::{c_char, CStr};

use cdump::{CDebug, CDeserialize, CSerialize};
use tests::eval_debug;

#[derive(CDebug, CSerialize, CDeserialize)]
#[repr(C)]
struct DeepFoo {
    len: u32,
    #[cdump(array(len = self.len))]
    b: *const ShallowBar,
    c: f64,
}

#[derive(Copy, Clone, CSerialize, CDeserialize, CDebug)]
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

    eval_debug(&obj);

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { DeepFoo::deserialize(&mut reader) };

    eval_debug(&copy);
    assert_eq!(obj.len, copy.len);
    assert_ne!(obj.b, copy.b);
    assert_eq!(unsafe { *obj.b }, unsafe { *copy.b });
    assert_eq!(unsafe { *obj.b.add(1) }, unsafe { *copy.b.add(1) });
}

#[derive(CDebug, CSerialize, CDeserialize)]
#[repr(C)]
struct ArrayOfPrimitives {
    len: u32,
    #[cdump(array(len = self.len))]
    b: *const u16,
}

#[test]
fn of_primitives() {
    let array: [u16; 5] = [45644, 2566, 3345, 8854, 12345];
    let obj = ArrayOfPrimitives {
        len: array.len() as u32,
        b: array.as_ptr(),
    };

    eval_debug(&obj);

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { ArrayOfPrimitives::deserialize(&mut reader) };

    eval_debug(&copy);
    assert_eq!(obj.len, copy.len);
    assert_ne!(obj.b, copy.b);
    for i in 0..array.len() {
        assert_eq!(unsafe { *obj.b.add(i) }, unsafe { *copy.b.add(i) });
    }
}

#[derive(CDebug, CSerialize, CDeserialize)]
#[repr(C)]
struct ArrayWithExpressionInLen {
    len: u8,
    #[cdump(array(len = self.len as usize / std::mem::size_of::<u32>()))]
    b: *const u32,
}

#[test]
fn expression_in_len() {
    let array: [u32; 5] = [567353453, 2352623, 457345353, 23525235, 2384279479];
    let obj = ArrayWithExpressionInLen {
        len: (array.len() * std::mem::size_of::<u32>()) as u8,
        b: array.as_ptr(),
    };

    eval_debug(&obj);

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { ArrayWithExpressionInLen::deserialize(&mut reader) };

    eval_debug(&copy);
    assert_eq!(obj.len, copy.len);
    assert_ne!(obj.b, copy.b);
    for i in 0..array.len() {
        assert_eq!(unsafe { *obj.b.add(i) }, unsafe { *copy.b.add(i) });
    }
}

#[derive(CDebug, CSerialize, CDeserialize)]
#[repr(C)]
struct ArrayOfPointers {
    len: u8,
    #[cdump(array(len = self.len))]
    data: *const *const ShallowBar,
}

#[test]
fn array_of_pointers() {
    let text1 = c"in that order of can-ing";
    let text2 = c"bad habits";

    let array = [
        ShallowBar {
            a: 4.7232552462,
            b: text1.as_ptr(),
            c: 1234515,
        },
        ShallowBar {
            a: -235252.5572352,
            b: text2.as_ptr(),
            c: 1212385628,
        },
    ];
    let ptrs: Vec<_> = array.iter().map(|x| x as *const _).collect();
    let obj = ArrayOfPointers {
        len: ptrs.len() as u8,
        data: ptrs.as_ptr(),
    };

    eval_debug(&obj);

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { ArrayOfPointers::deserialize(&mut reader) };

    eval_debug(&copy);
    assert_eq!(obj.len, copy.len);
    assert_ne!(obj.data, copy.data);
    for i in 0..ptrs.len() {
        assert_ne!(unsafe { obj.data.add(i) }, unsafe { copy.data.add(i) });
        assert_eq!(unsafe { **obj.data.add(i) }, unsafe { **copy.data.add(i) });
    }
}
