# cdump
[![crates.io](https://img.shields.io/crates/v/cdump)](https://crates.io/crates/cdump)
[![docs.rs](https://docs.rs/cdump/badge.svg)](https://docs.rs/cdump)

Crate for deep binary serialization of raw C types like e.g. [Vulkan structures](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkInstanceCreateInfo.html). Providen via macro with performance and throughput in mind.

> [!WARNING] 
> This crate focus on unsafe side of Rust, probably this is not a tool which you should use in your simple program which does not use [FFI](https://doc.rust-lang.org/nomicon/ffi.html). Be careful what you use.

### Support
- [x] [Shallow/plain data](docs/features/shallow.md)
- [x] [Deep serialization under single pointer](docs/features/deep.md)
- [x] [Arrays with providen length by another field](docs/features/array.md)
- [x] [CString](docs/features/cstring.md)
- [x] [Array of CStrings](docs/features/cstring_array.md)
- [x] [Dynamic types](docs/features/dynamic.md)

Read more in the [changelog](/CHANGELOG.md).

## Why?
For my [other project](https://github.com/Vixenka/wie) I had to serialize and deserialization raw C data providen by FFI to send them via sockets. Crates like [rkyv](https://crates.io/crates/rkyv) or [flatbuffers](https://crates.io/crates/flatbuffers) exists, but they mostly focus on serialization of Rust types, not C like. I could not find a crate for my case, so I created my own to fullfil a void. 

## Usage
```rust
use std::ffi::c_char;
use cdump::{CSerialize, CDeserialize};

#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    text: *const c_char,
    len_of_bars: u32,
    #[cdump(array(len = self.len_of_bars))]
    bars: *const Bar,
}

#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Bar {
    text_of_bar: *const c_char,
}

// Create object
let text = c"Hello world!";
let text_of_bar = c"Hello bar!";

let bars = [Bar {
    text_of_bar: text_of_bar.as_ptr(),
}];

let foo = Foo {
    text: text.as_ptr(),
    len_of_bars: bars.len() as u32,
    bars: bars.as_ptr(),
};

// Serialize and deserialize
let mut buf = cdump::CDumpBufferWriter::new(16);
// SAFETY: we upper initialize whole struct in this scope, which prevent data from dropping
unsafe { foo.serialize(&mut buf); }

let mut reader = buf.into_reader();
// SAFETY: reader's buffer contains Foo what is guaranteed above 
let copy_of_foo = unsafe { Foo::deserialize(&mut reader) };
```
More information about usage you can find at [support list](#support).

## License
cdump is licensed under the [MIT](/LICENSE) license.
