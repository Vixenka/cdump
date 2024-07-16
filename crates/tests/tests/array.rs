use cdump::{CDeserialize, CSerialize};

#[derive(Debug, CSerialize, CDeserialize)]
#[repr(C)]
struct DeepFoo {
    len: u32,
    #[cdump(array(len = len))]
    b: *const ShallowBar,
    c: f64,
}

#[derive(Debug, Copy, Clone, PartialEq, CSerialize, CDeserialize)]
#[repr(C)]
struct ShallowBar {
    a: f64,
    b: u8,
    c: u32,
}

#[test]
fn array() {
    let array = [
        ShallowBar {
            a: 19.84,
            b: 20,
            c: 1864,
        },
        ShallowBar {
            a: 20.77,
            b: 11,
            c: 7864,
        },
    ];
    let obj = DeepFoo {
        len: array.len() as u32,
        b: array.as_ptr(),
        c: 2024.07,
    };

    let mut buf = cdump::CDumpBufferWriter::new();
    obj.serialize(&mut buf);

    let mut reader = buf.into_reader();
    let copy = unsafe { DeepFoo::deserialize(&mut reader) };

    assert_eq!(obj.len, copy.len);
    assert_ne!(obj.b, copy.b);
    assert_eq!(unsafe { *obj.b }, unsafe { *copy.b });
    assert_eq!(unsafe { *obj.b.add(1) }, unsafe { *copy.b.add(1) });
}
