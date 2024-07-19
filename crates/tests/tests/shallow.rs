use cdump::{CDeserialize, CSerialize};

#[derive(Debug, CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    a: u32,
    b: f64,
}

#[test]
fn shallow() {
    let obj = Foo {
        a: 1984,
        b: 2024.06,
    };

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { Foo::deserialize(&mut reader) };

    assert_eq!(obj.a, copy.a);
    assert_eq!(obj.b, copy.b);
}
