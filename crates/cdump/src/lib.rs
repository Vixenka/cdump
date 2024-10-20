#![doc = include_str!("../../../README.md")]

use std::{mem, ptr};

#[cfg(feature = "builtin-buffer")]
use aligned_vec::AVec;
#[cfg(feature = "builtin-buffer")]
use std::cell::UnsafeCell;

pub use cdump_macro::{CDeserialize, CSerialize};
pub use memoffset::offset_of;
pub mod internal;

#[cfg(feature = "cdebug")]
pub use cdump_macro::CDebug;

/// Trait for buffer suitable for CSerialization.
/// # Safety
/// The implementor must ensure that the buffer is prepared for the serialization next objects.
///
/// Buffer must be properly aligned for the writing data to it, e.g. if any object or it part is aligned to 16 bytes,
/// then the first byte of the buffer must be also aligned to 16 bytes.
pub unsafe trait CDumpWriter {
    /// Align the buffer to the `T`.
    fn align<T>(&mut self);
    /// Push the slice to the buffer.
    fn push_slice(&mut self, slice: &[u8]);

    /// Get current length of the buffer.
    fn len(&self) -> usize;

    /// Check if the buffer is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get mutable pointer to the [`u8`] at the `index`.
    /// # Safety
    /// Caller must ensure that the `index` is a valid index in the buffer.
    unsafe fn as_mut_ptr_at(&mut self, index: usize) -> *mut u8;
}

/// Trait for buffer suitable for CDeserialization.
/// # Safety
/// The implementor must ensure that the buffer is prepared for the deserialization next objects.
///
/// First byte of the buffer must be aligned to the same alignment as the first byte of the original buffer used for
/// serialization.
pub unsafe trait CDumpReader {
    /// Align the buffer to the `T`.
    fn align<T>(&mut self);
    /// Adds the length to the current position.
    fn add_read(&mut self, len: usize);

    /// Read raw slice from the buffer, and returns the pointer to start of it.
    /// # Safety
    /// Caller must ensure that the `len` will not exceed the buffer length.
    unsafe fn read_raw_slice(&mut self, len: usize) -> *const u8;

    /// Read mutable reference to `T` which is located at index, without propagating the read count.
    /// # Safety
    /// The caller must ensure that the next data in the buffer from index is a valid representation of `T`, and must be careful
    /// with multiple mutable references to the same data.
    unsafe fn as_mut_ptr_at<T>(&self, index: usize) -> *mut T;

    fn get_read(&self) -> usize;
}

/// Trait for serializing the raw data to the buffer.
pub trait CSerialize<T: CDumpWriter> {
    /// Serialize the data to the buffer.
    /// # Safety
    /// The caller must ensure that the
    unsafe fn serialize(&self, buf: &mut T);

    /// Serializes the data to the buffer, ommiting the shallow copy.'
    /// # Params
    /// * `buf` - The buffer to write to.
    /// * `start_index` - The index in the buffer where shallow copied data of the object is located.
    /// # Safety
    /// Caller must ensure that the `start_index` is valid.
    unsafe fn serialize_without_shallow_copy(&self, buf: &mut T, start_index: usize);
}

/// Trait for deserializing the raw data from the buffer.
pub trait CDeserialize<T: CDumpReader>: Sized {
    /// Deserialize the data from the buffer to the initialized memory.
    /// # Remarks
    /// Copy the whole object tree to the destination memory with reuse of destination's pointers.
    /// When debug assertions are enabled, the function will try to check if the destination memory have sufficient
    /// size.
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of `Self`.
    /// Field `dst` can be uninitialized, then reading from it is undefined behavior.
    unsafe fn deserialize_to(buf: &mut T, dst: *mut Self);

    /// Deserializes the data from the buffer to the destination, ommiting the shallow copy.
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of deep part of `Self`.
    unsafe fn deserialize_to_without_shallow_copy(buf: &mut T, temp: *mut Self, dst: *mut Self);

    /// Deserialize the data from the buffer, and returns the reference to object which memory is located in the buffer.
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of `Self`.
    unsafe fn deserialize_ref_mut(buf: &mut T) -> &mut Self;

    /// Deserializes the data from the buffer to the destination, ommiting the shallow copy.
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of deep part of `Self`.
    unsafe fn deserialize_ref_mut_without_shallow_copy(buf: &mut T, dst: *mut Self);

    /// Deserialize the data from the buffer, and returns the reference to object which memory is located in the buffer.
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of `Self`.
    unsafe fn deserialize_ref(buf: &mut T) -> &Self {
        Self::deserialize_ref_mut(buf)
    }
}

macro_rules! impl_cserialize_cdeserialize {
    ($t:ident) => {
        impl<T: CDumpWriter> CSerialize<T> for $t {
            unsafe fn serialize(&self, buf: &mut T) {
                internal::align_writer::<T, Self>(buf);
                buf.push_slice(&self.to_ne_bytes());
            }

            unsafe fn serialize_without_shallow_copy(&self, _buf: &mut T, _start_index: usize) {}
        }

        impl<T: CDumpReader> CDeserialize<T> for $t {
            unsafe fn deserialize_to(buf: &mut T, dst: *mut Self) {
                let size = mem::size_of::<Self>();
                ptr::copy_nonoverlapping(
                    Self::deserialize_ref_mut(buf) as *mut _ as *mut u8,
                    dst as *mut u8,
                    size,
                )
            }

            unsafe fn deserialize_to_without_shallow_copy(
                _buf: &mut T,
                _temp: *mut Self,
                _dst: *mut Self,
            ) {
            }

            unsafe fn deserialize_ref_mut(buf: &mut T) -> &mut Self {
                internal::align_reader::<T, Self>(buf);
                let reference = buf.read_raw_slice(mem::size_of::<Self>());
                &mut *(reference as *mut Self)
            }

            unsafe fn deserialize_ref_mut_without_shallow_copy(_buf: &mut T, _dst: *mut Self) {}
        }
    };
}

impl_cserialize_cdeserialize!(u8);
impl_cserialize_cdeserialize!(u16);
impl_cserialize_cdeserialize!(u32);
impl_cserialize_cdeserialize!(u64);
impl_cserialize_cdeserialize!(u128);
impl_cserialize_cdeserialize!(usize);
impl_cserialize_cdeserialize!(i8);
impl_cserialize_cdeserialize!(i16);
impl_cserialize_cdeserialize!(i32);
impl_cserialize_cdeserialize!(i64);
impl_cserialize_cdeserialize!(i128);
impl_cserialize_cdeserialize!(isize);
impl_cserialize_cdeserialize!(f32);
impl_cserialize_cdeserialize!(f64);

/// Simple buffer writer for CSerialization.
#[cfg(feature = "builtin-buffer")]
pub struct CDumpBufferWriter {
    data: AVec<u8>,
}

#[cfg(feature = "builtin-buffer")]
impl CDumpBufferWriter {
    pub fn new(align: usize) -> Self {
        Self {
            data: AVec::new(align),
        }
    }

    pub fn into_reader(self) -> CDumpBufferReader {
        CDumpBufferReader::new(self.data)
    }
}

#[cfg(feature = "builtin-buffer")]
impl From<CDumpBufferWriter> for AVec<u8> {
    fn from(writer: CDumpBufferWriter) -> Self {
        writer.data
    }
}

#[cfg(feature = "builtin-buffer")]
unsafe impl CDumpWriter for CDumpBufferWriter {
    fn align<T>(&mut self) {
        let m = self.data.len() % mem::align_of::<T>();
        if m == 0 {
            return;
        }

        let missing = mem::align_of::<T>() - m;
        self.data.resize(self.data.len() + missing, 0);

        debug_assert_eq!(0, self.data.len() % mem::align_of::<T>());
    }

    fn push_slice(&mut self, slice: &[u8]) {
        self.data.extend_from_slice(slice);
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    unsafe fn as_mut_ptr_at(&mut self, index: usize) -> *mut u8 {
        self.data.as_mut_ptr().add(index)
    }
}

/// Simple buffer reader for CDeserialization.
#[cfg(feature = "builtin-buffer")]
pub struct CDumpBufferReader {
    data: UnsafeCell<AVec<u8>>,
    read: usize,
}

#[cfg(feature = "builtin-buffer")]
impl CDumpBufferReader {
    pub fn new(data: AVec<u8>) -> Self {
        Self {
            data: UnsafeCell::new(data),
            read: 0,
        }
    }
}

#[cfg(feature = "builtin-buffer")]
unsafe impl CDumpReader for CDumpBufferReader {
    fn align<T>(&mut self) {
        let m = self.read % mem::align_of::<T>();
        if m != 0 {
            self.read += mem::align_of::<T>() - m;
        }
        debug_assert_eq!(0, self.read % mem::align_of::<T>());
    }

    unsafe fn read_raw_slice(&mut self, len: usize) -> *const u8 {
        let s = &*self.data.get();
        let ptr = s.as_ptr().add(self.read);
        self.read += len;
        ptr
    }

    fn add_read(&mut self, len: usize) {
        self.read += len;
    }

    unsafe fn as_mut_ptr_at<T>(&self, index: usize) -> *mut T {
        let s = &mut *self.data.get();
        s.as_mut_ptr().add(index) as *mut T
    }

    fn get_read(&self) -> usize {
        self.read
    }
}
