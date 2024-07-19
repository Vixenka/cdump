#![doc = include_str!("../../../README.md")]

use std::mem::{self, MaybeUninit};

#[cfg(feature = "builtin-buffer")]
use aligned_vec::AVec;
#[cfg(feature = "builtin-buffer")]
use std::cell::UnsafeCell;

pub use cdump_macro::{CDeserialize, CSerialize};
pub use memoffset::offset_of;
pub mod internal;

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
    /// Read the slice from the buffer.
    fn read_slice(&mut self, len: usize) -> &[u8];
    /// Adds the length to the current position.
    fn add_read(&mut self, len: usize);

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
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of `Self`.
    /// Field `dst` can be uninitialized, then reading from it is undefined behavior.
    unsafe fn deserialize_to(buf: &mut T, dst: &mut Self);

    /// Deserializes the data from the buffer to the destination, ommiting the shallow copy.
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of deep part of `Self`.
    unsafe fn deserialize_to_without_shallow_copy(buf: &mut T, dst: &mut Self);

    /// Deserialize the data from the buffer to the uninitialized memory.
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of `Self`.
    unsafe fn deserialize_to_uninit(buf: &mut T, dst: &mut MaybeUninit<Self>) {
        // Safety: MaybeUninit<T> is a transparent wrapper around T, so it should be work properly.
        Self::deserialize_to(
            buf,
            mem::transmute::<&mut MaybeUninit<Self>, &mut Self>(dst),
        );
    }

    /// Deserialize the data from the buffer to the new object of `Self`.
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of `Self`.
    unsafe fn deserialize(buf: &mut T) -> Self {
        let mut dst = MaybeUninit::uninit();
        Self::deserialize_to_uninit(buf, &mut dst);
        // Safety: `dst` should be fully initialized via [`deserialize_to_uninit`].
        unsafe { dst.assume_init() }
    }
}

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

    fn read_slice(&mut self, len: usize) -> &[u8] {
        let slice = &self.data.get_mut()[self.read..self.read + len];
        self.read += len;
        slice
    }

    fn add_read(&mut self, len: usize) {
        self.read += len;
    }

    unsafe fn as_mut_ptr_at<T>(&self, index: usize) -> *mut T {
        let s = &mut *self.data.get();
        (&mut s[index]) as *mut u8 as *mut T
    }

    fn get_read(&self) -> usize {
        self.read
    }
}
