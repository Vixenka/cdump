use std::ffi::c_char;

use crate::CDeserialize;

/// Get the length of the C string.
/// # Safety
/// Caller has provided a pointer to a valid C string.
/// # Remarks
/// This function is a wrapper around `libc::strlen`. It exists so that a program that uses this crate does not have to
/// reference to libc.
#[inline]
pub unsafe fn libc_strlen(cs: *const c_char) -> usize {
    libc::strlen(cs)
}

/// Set the `len` in the `index` of the buffer.
/// # Safety
/// Caller must ensure that index is a valid index in the buffer.
#[inline]
pub unsafe fn set_length_in_ptr<T>(buf: &mut T, index: usize, len: usize)
where
    T: crate::CDumpWriter,
{
    *(buf.get_mut(index) as *mut _ as *mut usize) = len;
}

/// Deserialize the shallow copied data in the buffer and returns the reference to it.
/// # Safety
/// Caller must ensure that the next data in the buffer is a valid representation of `T2`.
pub unsafe fn deserialize_shallow_copied<T1, T2>(buf: &mut T1) -> *mut T2
where
    T1: crate::CDumpReader,
    T2: crate::CDeserialize<T1>,
{
    buf.align::<T2>();
    let reference = &mut *buf.read_mut::<T2>();
    buf.add_read(::std::mem::size_of::<T2>());
    CDeserialize::deserialize_to_without_shallow_copy(buf, reference);
    reference
}

// Deserialize the shallow copied data in the buffer and returns the reference to it.
/// # Safety
/// Caller must ensure that the next data in the buffer is a valid representation of `T2`.
#[inline]
pub unsafe fn deserialize_shallow_copied_at<T1, T2>(buf: &mut T1, index: usize) -> *mut T2
where
    T1: crate::CDumpReader,
    T2: crate::CDeserialize<T1>,
{
    let reference = &mut *buf.get_mut(index);
    CDeserialize::deserialize_to_without_shallow_copy(buf, reference);
    reference
}
