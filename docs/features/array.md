# Array serialization

Objects can point via pointers to C like arrays. Pointer looks exactly like in [deep serialization](deep.md), but must lineary store `N` occurrences of object.

`N` is definied via another field of struct which contains integer value selected by special attribute. 

## Usage
Create two structures with C layout, and derive to them macros. First structure `Foo` is pointing to the N instances of type `Bar`. `N` is equal to value under field named `length`. Const and mutable pointer is allowed.
```rust
#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    length: u32,
    #[cdump(array(len = length))]
    ptr: *const Bar,
}

#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Bar {
    hello: i32,
    world: f32,
}
```

## Safety
Pointer to object must be valid.
