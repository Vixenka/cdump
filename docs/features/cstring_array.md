# Serialization of array with C strings

This type of serialization is connection of [array](array.md) and [C string](cstring.md) serialization. First layer of pointer points to array of pointers, and then every pointer point to C string.

## Usage
Create valid memory of C strings, create array with pointers to that C strings, and then put length of array to field `length_of_texts`, and set `texts` pointer to beginning of array.
```rust
#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    length_of_texts: u16,
    #[cdump(array(len = length_of_texts))]
    texts: *const *const c_char,
}
```

## Safety
Every pointer must be valid, and not null.
