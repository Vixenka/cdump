# CDebug
Macro to provide Rust's [Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html) trait for raw C types.

## Usage
Extend any previous struct by adding next `derrive` with `CDebug` argument.
```rust
#[derive(CSerialize, CDeserialize, CDebug)]
#[repr(C)]
struct Foo {
    length: u32,
    #[cdump(array(len = self.length))]
    ptr: *const u8,
}
```
> Derrive macro CDebug conflicts with Rust's Debug macro. The second one must be removed.

## Safety
Every pointer must be valid or be null.
