use std::ffi::{c_char, c_void, CStr};

use cdump::{CDeserialize, CDumpReader, CDumpWriter, CSerialize};

#[derive(Debug, CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    a: u32,
    #[cdump(dynamic(serializer = custom_serializer, deserializer = custom_deserializer))]
    d: *const c_void,
    text: *const c_char,
}

#[derive(Debug, CSerialize, CDeserialize)]
#[repr(C)]
struct DynamicBar {
    ty: DynamicType,
    text: *const c_char,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
enum DynamicType {
    Bar = 1,
}

#[test]
fn dynamic() {
    let dynamic_text = c"Never coming back!";
    let dynamic = DynamicBar {
        ty: DynamicType::Bar,
        text: dynamic_text.as_ptr(),
    };
    let text = c"Hello world!";
    let obj = Foo {
        a: 1984,
        d: &dynamic as *const _ as *const c_void,
        text: text.as_ptr(),
    };

    let mut buf = cdump::CDumpBufferWriter::new();
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { Foo::deserialize(&mut reader) };

    assert_eq!(obj.a, copy.a);
    assert_ne!(obj.d, copy.d);
    unsafe {
        assert_eq!(
            (*(obj.d as *const DynamicBar)).ty,
            (*(copy.d as *const DynamicBar)).ty
        );
        assert_eq!(
            CStr::from_ptr((*(obj.d as *const DynamicBar)).text),
            CStr::from_ptr((*(copy.d as *const DynamicBar)).text)
        );
    }
    assert_ne!(obj.text, copy.text);
    unsafe {
        assert_eq!(CStr::from_ptr(obj.text), CStr::from_ptr(copy.text));
    }
}

unsafe fn custom_serializer<T: CDumpWriter>(buf: &mut T, obj: *const c_void) {
    buf.align::<DynamicType>();
    let ty = *(obj as *const DynamicType);
    match ty {
        DynamicType::Bar => {
            let obj = &*(obj as *const DynamicBar);
            obj.serialize(buf)
        }
    }
}

unsafe fn custom_deserializer<T: CDumpReader>(buf: &mut T) -> *const c_void {
    buf.align::<DynamicType>();
    let ptr = buf.as_mut_ptr_at::<c_void>(buf.get_read());
    let ty = *(ptr as *const DynamicType);
    match ty {
        DynamicType::Bar => {
            let dst = &mut *(ptr as *mut DynamicBar);
            buf.add_read(::std::mem::size_of::<DynamicBar>());
            DynamicBar::deserialize_to_without_shallow_copy(buf, dst);
        }
    }
    ptr
}
