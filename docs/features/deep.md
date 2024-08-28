# Deep serialization

When object is not [shallow](shallow.md), it have pointers to another objects which create a tree.

## Usage
Create two structures with C layout, and derive to them macros. First structure `Foo` is pointing to instance of type `Bar`. Const and mutable pointer is allowed.
```rust
#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
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
Pointer to object must be valid or null.
