use std::ffi::c_char;

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
