# Serialization of C strings

Objects can contain pointers of `c_char` which point to C like strings. End of string is providen via null character terminator (`\0`).

## Usage
Create valid C string memory, and provide it to field of struct.
```rust
#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    text: *const c_char,
}
```

## Safety
Pointer must point to valid C string which length does not exceed `usize`.
