use std::{
    ffi::{c_char, c_void, CStr},
    fmt::Debug,
    mem,
};

use cdump::{CDebug, CDeserialize, CDumpReader, CDumpWriter, CSerialize};
use tests::eval_debug;

#[derive(CDebug, CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    a: u32,
    #[cdump(dynamic(serializer = custom_serializer, deserializer = custom_deserializer, cdebugger = custom_cdebugger))]
    d: *const c_void,
    text: *const c_char,
}

#[derive(CDebug, CSerialize, CDeserialize)]
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

    eval_debug(&obj);

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { Foo::deserialize(&mut reader) };

    eval_debug(&copy);
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

unsafe fn custom_serializer<T: CDumpWriter>(buf: &mut T, obj: *const c_void) -> usize {
    buf.align::<DynamicBar>();
    let ty = *(obj as *const DynamicType);
    match ty {
        DynamicType::Bar => {
            let obj = &*(obj as *const DynamicBar);
            obj.serialize(buf);
            mem::size_of::<DynamicBar>()
        }
    }
}

unsafe fn custom_deserializer<T: CDumpReader>(buf: &mut T) -> *const c_void {
    buf.align::<DynamicBar>();
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

unsafe fn custom_cdebugger(obj: *const c_void) -> &'static dyn Debug {
    let ty = *(obj as *const DynamicType);
    match ty {
        DynamicType::Bar => &*(obj as *const DynamicBar),
    }
}

#[derive(CSerialize, CDeserialize, CDebug)]
#[repr(C)]
struct ArrayOfDynamicTypes {
    len: u8,
    #[cdump(array(len = self.len))]
    #[cdump(dynamic(serializer = custom_serializer, deserializer = custom_deserializer))]
    data: *const *const c_void,
}

#[test]
fn array_of_dynamic_types() {
    let text1 = c"Never coming back!";
    let text2 = c"Hello world!";

    let array = [
        DynamicBar {
            ty: DynamicType::Bar,
            text: text1.as_ptr(),
        },
        DynamicBar {
            ty: DynamicType::Bar,
            text: text2.as_ptr(),
        },
    ];
    let ptrs = array
        .iter()
        .map(|x| x as *const _ as *const c_void)
        .collect::<Vec<_>>();

    let obj = ArrayOfDynamicTypes {
        len: ptrs.len() as u8,
        data: ptrs.as_ptr(),
    };

    eval_debug(&obj);

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { ArrayOfDynamicTypes::deserialize(&mut reader) };

    eval_debug(&copy);

    assert_eq!(obj.len, copy.len);
    assert_ne!(obj.data, copy.data);
    for i in 0..ptrs.len() {
        assert_ne!(unsafe { *obj.data.add(i) }, unsafe { *copy.data.add(i) });
        unsafe {
            assert_eq!(
                (**(obj.data.add(i) as *const *const DynamicBar)).ty,
                (**(copy.data.add(i) as *const *const DynamicBar)).ty
            );
            assert_eq!(
                CStr::from_ptr((**(obj.data.add(i) as *const *const DynamicBar)).text),
                CStr::from_ptr((**(copy.data.add(i) as *const *const DynamicBar)).text)
            );
        }
    }
}
