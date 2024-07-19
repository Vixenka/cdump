# Serialization of shallow/plain data

This part of serialization is responsible for copying data under pointer to origin object, and save that to the buffer without any processing.

## Usage
Create structure with C layout (via repr C), and attach derive attribute to implement functions via macros.
```rust
#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    example: i32,
    of: f64,
    plain: u8,
    data: usize
}
```

## Safety
Definition, layout, endianness and size of serialized object, and deserialized object must be exactly the same.
