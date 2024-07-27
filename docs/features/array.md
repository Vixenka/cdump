# Array serialization

Objects can point via pointers to C like arrays. Pointer looks exactly like in [deep serialization](deep.md), but must lineary store `N` occurrences of object.

`N` is any expression in Rust, e.g. another field of struct which contains integer value selected by special attribute.
Current object is avaiable under `self`.

## Usage
Create two structures with C layout, and derive to them macros. First structure `Foo` is pointing to the N instances of type `Bar`. `N` is equal to value under field named `length`. Const and mutable pointer is allowed.
```rust
#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    length: u32,
    #[cdump(array(len = self.length))]
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
Pointer to object must be valid. Expression for length contain `self` object which can be not fully initialized memory.
