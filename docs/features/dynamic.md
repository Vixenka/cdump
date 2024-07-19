# Serialization of dynamic types

Sometimes struct have fields which are dynamic, and its size or type is not knowed at compile time. For that cdump provide functionality which allow user to define their own serializator.

## Usage
Create struct with pointer which object is not knowed at compilation time, and add it attribute which specify path to serialization and deseriazation functions:
```rust
#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Foo {
    #[cdump(dynamic(serializer = custom_serializer, deserializer = custom_deserializer))]
    ptr: *const c_void,
}
```

Then create object with can suit to that pointer:
```rust
#[derive(CSerialize, CDeserialize)]
#[repr(C)]
struct Bar {
    text: *const c_char,
}
```

Finally create `custom_serializer`, and `custom_deserializer`:
```rust
unsafe fn custom_serializer<T: CDumpWriter>(buf: &mut T, obj: *const c_void) {
    /// Align buffer to our type.
    buf.align::<Bar>();
    /// Cast object to our type, and serialize it.
    let obj = &*(obj as *const Bar);
    obj.serialize(buf)
}

unsafe fn custom_deserializer<T: CDumpReader>(buf: &mut T) -> *const c_void {
    /// Align buffer to our type.
    buf.align::<Bar>();

    /// Read next data as pointer without propagate read count.
    let ptr = buf.read_mut::<c_void>();

    // Cast pointer to our type.
    let dst = &mut *(ptr as *mut Bar);
    // Propagate read buffer by size of our type.
    buf.add_read(mem::size_of::<Bar>());
    // Deserialize data from buffer to pointer without reading shallow data. This is because under our pointer data of shallow copy already exists.
    Bar::deserialize_to_without_shallow_copy(buf, dst);

    // Return pointer to the object.
    ptr
}
```

## Safety
In that method of serialization many things are related to what user of crate will do. Remember to have everything correctly aligned, and to do not creating invalid states.
