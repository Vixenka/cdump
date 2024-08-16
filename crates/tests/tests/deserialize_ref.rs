use cdump::{CDebug, CDeserialize, CSerialize};
use tests::eval_debug;

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

#[derive(CDebug, Copy, Clone, PartialEq, CSerialize, CDeserialize)]
#[repr(C)]
struct ShallowBar {
    a: f64,
    b: u8,
    c: u32,
}

#[test]
fn deserialize_ref() {
    let bar = ShallowBar {
        a: 19.84,
        b: 20,
        c: 1864,
    };
    let bar2 = ShallowBar {
        a: 20.77,
        b: 11,
        c: 7864,
    };
    let obj = DeepFoo {
        a: 19,
        b: &bar,
        c: 2024.06,
        d: &bar2,
    };

    eval_debug(&obj);

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { DeepFoo::deserialize_ref(&mut reader) };

    eval_debug(&copy);
    assert_eq!(obj.a, copy.a);
    assert_ne!(obj.b, copy.b);
    assert_eq!(unsafe { *obj.b }, unsafe { *copy.b });
    assert_eq!(obj.c, copy.c);
    assert_ne!(obj.d, copy.d);
    assert_eq!(unsafe { *obj.d }, unsafe { *copy.d });
}
