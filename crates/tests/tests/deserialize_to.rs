use std::{ffi::c_void, mem};

use cdump::{CDebug, CDeserialize, CDumpReader, CDumpWriter, CSerialize};

#[derive(Copy, Clone, CSerialize, CDeserialize, CDebug)]
#[repr(C)]
struct DeepFoo {
    a: u8,
    b: *const ShallowBar,
    c: f32,
    d: *const ShallowBar,
}

impl PartialEq for DeepFoo {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a
            && unsafe { *self.b == *other.b }
            && self.c == other.c
            && unsafe { *self.d == *other.d }
    }
}

#[derive(CDebug, Copy, Clone, CSerialize, CDeserialize)]
#[repr(C)]
struct ShallowBar {
    a: f64,
    b: u8,
    c: u32,
    #[cdump(dynamic(serializer = custom_serializer, deserializer = custom_deserializer, size_of = custom_sizeof))]
    ptr: *const c_void,
}

impl PartialEq for ShallowBar {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a
            && self.b == other.b
            && self.c == other.c
            && unsafe { *(self.ptr as *const Dynamic) == *(other.ptr as *const Dynamic) }
    }
}

#[derive(CDebug, PartialEq, CSerialize, CDeserialize)]
#[repr(C)]
struct Dynamic {
    active: bool,
}

unsafe fn custom_serializer<T: CDumpWriter>(buf: &mut T, obj: *const c_void) {
    (*(obj as *const Dynamic)).serialize(buf);
}

unsafe fn custom_deserializer<T: CDumpReader>(buf: &mut T) -> (*const c_void, usize) {
    (
        Dynamic::deserialize_ref(buf) as *const _ as *const c_void,
        mem::size_of::<Dynamic>(),
    )
}

unsafe fn custom_sizeof(_obj: *const c_void) -> usize {
    mem::size_of::<Dynamic>()
}

#[test]
fn copy_to_original_memory_tree() {
    let dynamic1 = Dynamic { active: true };
    let shallow1 = ShallowBar {
        a: 19.84,
        b: 20,
        c: 1864,
        ptr: &dynamic1 as *const _ as *const c_void,
    };
    let dynamic2 = Dynamic { active: true };
    let shallow2 = ShallowBar {
        a: 20.77,
        b: 11,
        c: 7864,
        ptr: &dynamic2 as *const _ as *const c_void,
    };
    let mut deep_foo = DeepFoo {
        a: 19,
        b: &shallow1,
        c: 2024.08,
        d: &shallow2,
    };

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe {
        deep_foo.serialize(&mut buf);
        deep_foo.serialize(&mut buf);
        deep_foo.serialize(&mut buf);
    }

    let mut reader = buf.into_reader();
    let copy = unsafe { DeepFoo::deserialize_ref(&mut reader) };
    compare_copy(&deep_foo, copy);

    unsafe { DeepFoo::deserialize_to(&mut reader, &mut deep_foo) };

    let copy = unsafe { DeepFoo::deserialize_ref(&mut reader) };
    compare_copy(&deep_foo, copy);

    assert_eq!(&shallow1 as *const _, deep_foo.b);
    assert_eq!(&dynamic1 as *const _, unsafe {
        (*deep_foo.b).ptr as *const Dynamic
    });
    assert_eq!(&shallow2 as *const _, deep_foo.d);
    assert_eq!(&dynamic1 as *const _, unsafe {
        (*deep_foo.d).ptr as *const Dynamic
    });
}

fn compare_copy(original: &DeepFoo, copy: &DeepFoo) {
    assert_eq!(original.a, copy.a);
    assert_ne!(original.b, copy.b);
    assert_eq!(unsafe { *original.b }, unsafe { *copy.b });
    assert_ne!(unsafe { *original.b }.ptr, unsafe { *copy.b }.ptr);
    assert_eq!(original.c, copy.c);
    assert_ne!(original.d, copy.d);
    assert_eq!(unsafe { *original.d }, unsafe { *copy.d });
    assert_ne!(unsafe { *original.d }.ptr, unsafe { *copy.d }.ptr);
}
