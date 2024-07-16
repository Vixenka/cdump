use cdump::{CDeserialize, CSerialize};

#[derive(Debug, CSerialize, CDeserialize)]
#[repr(C)]
struct DeepFoo {
    a: u8,
    b: *const ShallowBar,
    c: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, CSerialize, CDeserialize)]
#[repr(C)]
struct ShallowBar {
    a: f64,
    b: u8,
    c: u32,
}

#[test]
fn deep_const() {
    let bar = ShallowBar {
        a: 19.84,
        b: 20,
        c: 1864,
    };
    let obj = DeepFoo {
        a: 19,
        b: &bar,
        c: 2024.06,
    };

    let mut buf = cdump::CDumpBufferWriter::new();
    obj.serialize(&mut buf);

    let mut reader = buf.into_reader();
    let copy = unsafe { DeepFoo::deserialize(&mut reader) };

    assert_eq!(obj.a, copy.a);
    assert_ne!(obj.b, copy.b);
    assert_eq!(unsafe { *obj.b }, unsafe { *copy.b });
    assert_eq!(obj.c, copy.c);
}
