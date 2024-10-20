use std::{ffi::c_void, mem};

use cdump::{CDebug, CDeserialize, CDumpBufferReader, CDumpReader, CDumpWriter, CSerialize};
use tests::eval_debug;

#[derive(Copy, Clone, CSerialize, CDeserialize, CDebug)]
#[repr(C)]
struct DeepFoo {
    a: u8,
    b: *mut ShallowBar,
    c: f32,
    d: *mut ShallowBar,
    number_ptr: *mut u32,
    array_primitive_len: u32,
    #[cdump(array(len = self.array_primitive_len))]
    array_primitive: *mut u16,
}

#[derive(CDebug, Copy, Clone, CSerialize, CDeserialize)]
#[repr(C)]
struct ShallowBar {
    a: f64,
    b: u8,
    c: u32,
    #[cdump(dynamic(serializer = custom_serializer, deserializer = custom_deserializer, size_of = custom_sizeof))]
    ptr: *mut c_void,
}

#[derive(Clone, Copy, CDebug, PartialEq, CSerialize, CDeserialize)]
#[repr(C)]
struct Dynamic {
    value: u8,
}

unsafe fn custom_serializer<T: CDumpWriter>(buf: &mut T, obj: *const c_void) {
    (*(obj as *const Dynamic)).serialize(buf);
}

unsafe fn custom_deserializer<T: CDumpReader>(buf: &mut T) -> (*mut c_void, usize) {
    (
        Dynamic::deserialize_ref_mut(buf) as *mut _ as *mut c_void,
        mem::size_of::<Dynamic>(),
    )
}

unsafe fn custom_sizeof(_obj: *const c_void) -> usize {
    mem::size_of::<Dynamic>()
}

#[test]
fn deserialize_to_original_memory_tree() {
    let mut dynamic1 = Dynamic { value: 0 };
    let mut shallow1 = ShallowBar {
        a: 0.0,
        b: 0,
        c: 0,
        ptr: &mut dynamic1 as *mut _ as *mut c_void,
    };
    let mut dynamic2 = Dynamic { value: 0 };
    let mut shallow2 = ShallowBar {
        a: 0.0,
        b: 0,
        c: 0,
        ptr: &mut dynamic2 as *mut _ as *mut c_void,
    };
    let mut number_ptr = 0;
    let mut array_primitive = [0u16; 10];
    let mut deep_foo = DeepFoo {
        a: 0,
        b: &mut shallow1,
        c: 0.0,
        d: &mut shallow2,
        number_ptr: &mut number_ptr,
        array_primitive_len: array_primitive.len() as u32,
        array_primitive: array_primitive.as_mut_ptr(),
    };

    let mut reader = prepare_buffer();
    unsafe {
        DeepFoo::deserialize_to(&mut reader, &mut deep_foo);
    }

    eval_debug(&deep_foo);

    assert_eq!(deep_foo.a, 19);
    assert_eq!(deep_foo.b, &mut shallow1 as *mut _);
    assert_eq!(deep_foo.c, 2024.08);
    assert_eq!(deep_foo.d, &mut shallow2 as *mut _);
    assert_eq!(deep_foo.number_ptr, &mut number_ptr as *mut _);
    assert_eq!(number_ptr, 0x13);
    assert_eq!(deep_foo.array_primitive_len, 4);
    assert_eq!(deep_foo.array_primitive, array_primitive.as_mut_ptr());

    assert_eq!(shallow1.a, 19.84);
    assert_eq!(shallow1.b, 20);
    assert_eq!(shallow1.c, 1864);
    assert_eq!(shallow1.ptr, &mut dynamic1 as *mut _ as *mut c_void);

    assert_eq!(shallow2.a, 20.77);
    assert_eq!(shallow2.b, 11);
    assert_eq!(shallow2.c, 7864);
    assert_eq!(shallow2.ptr, &mut dynamic2 as *mut _ as *mut c_void);

    assert_eq!(dynamic1.value, 10);
    assert_eq!(dynamic2.value, 128);
}

fn prepare_buffer() -> CDumpBufferReader {
    let mut dynamic1 = Dynamic { value: 10 };
    let mut shallow1 = ShallowBar {
        a: 19.84,
        b: 20,
        c: 1864,
        ptr: &mut dynamic1 as *mut _ as *mut c_void,
    };
    let mut dynamic2 = Dynamic { value: 128 };
    let mut shallow2 = ShallowBar {
        a: 20.77,
        b: 11,
        c: 7864,
        ptr: &mut dynamic2 as *mut _ as *mut c_void,
    };
    let mut number_ptr = 0x13;
    let mut array_primitive = [5u16, 7u16, 13u16, 0u16];
    let deep_foo = DeepFoo {
        a: 19,
        b: &mut shallow1,
        c: 2024.08,
        d: &mut shallow2,
        number_ptr: &mut number_ptr,
        array_primitive_len: array_primitive.len() as u32,
        array_primitive: array_primitive.as_mut_ptr(),
    };

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe {
        deep_foo.serialize(&mut buf);
    }
    buf.into_reader()
}
