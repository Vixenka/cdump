use std::{ffi::c_void, fmt::Debug, ptr};

use cdump::{CDumpReader, CDumpWriter};

pub fn empty_serializer<T: CDumpWriter>(_buf: &mut T, _obj: *const c_void) {}

pub fn empty_deserializer<T: CDumpReader>(_buf: &mut T) -> (*mut c_void, usize) {
    (ptr::null_mut(), 0)
}

pub fn empty_sizeof(_obj: *const c_void) -> usize {
    0
}

pub fn eval_debug<T>(obj: &T)
where
    T: Debug,
{
    println!("{:?}", obj);
}
