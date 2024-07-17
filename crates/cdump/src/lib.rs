use std::{
    cell::UnsafeCell,
    mem::{self, MaybeUninit},
};

pub use cdump_macro::{dynamic_serializator, CDeserialize, CSerialize};
pub use memoffset::offset_of;
pub mod internal;

/// Trait for buffer suitable for CSerialization.
pub trait CDumpWriter {
    /// Align the buffer to the `T`.
    fn align<T>(&mut self);
    /// Push the slice to the buffer.
    fn push_slice(&mut self, slice: &[u8]);
    /// Get the mutable reference to the buffer at the index.
    fn get_mut(&mut self, index: usize) -> &mut u8;

    /// Get current length of the buffer.
    fn len(&self) -> usize;

    /// Check if the buffer is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Trait for buffer suitable for CDeserialization.
pub trait CDumpReader {
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
    unsafe fn get_mut<T>(&self, index: usize) -> *mut T;

    /// Read mutable reference to `T` which is located at the current position, without propagating the read count.
    /// # Safety
    /// The caller must ensure that the next data in the buffer is a valid representation of `T`, and must be careful
    /// with multiple mutable references to the same data.
    unsafe fn read_mut<T>(&self) -> *mut T;

    fn get_read(&self) -> usize;
}

/// Trait for serializing the raw data to the buffer.
pub trait CSerialize<T: CDumpWriter> {
    /// Serialize the data to the buffer.
    fn serialize(&self, buf: &mut T);

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
pub struct CDumpBufferWriter {
    data: Vec<u8>,
}

impl CDumpBufferWriter {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn into_reader(self) -> CDumpBufferReader {
        CDumpBufferReader::new(self.data)
    }
}

impl Default for CDumpBufferWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl From<CDumpBufferWriter> for Vec<u8> {
    fn from(writer: CDumpBufferWriter) -> Self {
        writer.data
    }
}

impl CDumpWriter for CDumpBufferWriter {
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

    fn get_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.data[index]
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

/// Simple buffer reader for CDeserialization.
pub struct CDumpBufferReader {
    data: UnsafeCell<Vec<u8>>,
    read: usize,
}

impl CDumpBufferReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: UnsafeCell::new(data),
            read: 0,
        }
    }
}

impl CDumpReader for CDumpBufferReader {
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

    unsafe fn get_mut<T>(&self, index: usize) -> *mut T {
        let s = &mut *self.data.get();
        (&mut s[index]) as *mut u8 as *mut T
    }

    unsafe fn read_mut<T>(&self) -> *mut T {
        self.get_mut(self.read)
    }

    fn get_read(&self) -> usize {
        self.read
    }
}
