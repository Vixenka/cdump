use cdump::{CDeserialize, CDumpBufferWriter, CSerialize};

#[test]
fn builtin_deserialize_to() {
    let initial = [33, 1984, 11, 6];

    let mut buf = CDumpBufferWriter::new(16);
    for value in initial {
        unsafe { value.serialize(&mut buf) };
    }

    let mut reader = buf.into_reader();
    let mut dst = [0u32; 4];
    for i in 0..dst.len() {
        unsafe { u32::deserialize_to(&mut reader, dst.as_mut_ptr().add(i)) };
    }

    for (a, b) in initial.iter().zip(dst.iter()) {
        assert_eq!(*a, *b);
    }
}
