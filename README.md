# cdump
[![crates.io](https://img.shields.io/crates/v/cdump)](https://crates.io/crates/cdump)
[![docs.rs](https://docs.rs/cdump/badge.svg)](https://docs.rs/cdump)

Crate for deep binary serialization of raw C types like e.g. [Vulkan structures](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkInstanceCreateInfo.html). Providen via macro with performance and throughput in mind.

> [!WARNING] 
> This crate focus on unsafe side of Rust, probably this is not a tool which you should use in your simple program which does not use [FFI](https://doc.rust-lang.org/nomicon/ffi.html). Be careful what you use.

### Support
- [x] Shallow/plain data
- [x] Deep serialization under single pointer
- [x] Arrays with providen length by another field
- [x] CString
- [x] Array of CStrings
- [x] Dynamic types

Read more in the [changelog](/CHANGELOG.md).

## Why?
For my [other project](https://github.com/Vixenka/wie) I had to serialize and deserialization raw C data providen by FFI to send them via sockets. Crates like [rkyv](https://crates.io/crates/rkyv) or [flatbuffers](https://crates.io/crates/flatbuffers) exists, but they mostly focus on serialization of Rust types, not C like. I could not find a crate for my case, so I created my own to fullfil a void. 

## Usage
```rust
#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    text: *const c_char,
    len_of_bars: u32,
    #[cdump(array(len = len_of_bars))]
    bars: *const Bar,
}

#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Bar {
    text_of_bar: *const c_char,
}
```
```rust
let mut buf = cdump::CDumpBufferWriter::new();
foo.serialize(&mut buf);

let mut reader = buf.into_reader();
// SAFETY: buffer of reader contains Foo what is guaranteed above 
let copy_of_foo = unsafe { DeepFoo::deserialize(&mut reader) };
```
More information about usage you can find at [support list](#support).

## License
cdump is licensed under the [MIT](/LICENSE) license.
