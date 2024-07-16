use field_analysis::{Field, FieldType};
use helpers::{validate_repr, ErrorExt};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

mod field_analysis;
mod helpers;

#[proc_macro_derive(CSerialize, attributes(cdump))]
pub fn c_serialize_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let validate_repr = validate_repr(&ast.attrs, "C", ast.span()).to_compile_error();

    let name = ast.ident.clone();
    let push_copy = push_copy();

    let deep_fields = match field_analysis::get_fields(&ast) {
        Ok(fields) => write_deep_fields(fields),
        Err(err) => err.to_compile_error(),
    };

    proc_macro::TokenStream::from(quote! {
        impl<T: ::cdump::CDumpWriter> ::cdump::CSerialize<T> for #name {
            fn serialize(&self, buf: &mut T) {
                #validate_repr

                buf.align::<Self>();
                let start_index = buf.len();
                #push_copy
                unsafe { self.serialize_without_shallow_copy(buf, start_index); }
            }

            unsafe fn serialize_without_shallow_copy(&self, buf: &mut T, start_index: usize) {
                #deep_fields
            }
        }
    })
}

fn push_copy() -> TokenStream {
    quote! {
        buf.push_slice(unsafe {
            std::slice::from_raw_parts(self as *const _ as *const u8, std::mem::size_of::<Self>())
        });
    }
}

fn write_deep_fields(fields: Vec<Field>) -> TokenStream {
    let mut start_index = false;
    let mut quotes = Vec::new();

    for field in fields {
        quotes.push(write_deep_fields_inner(&mut start_index, &field, None));
    }

    quotes.into_iter().collect()
}

fn write_deep_fields_inner(
    start_index: &mut bool,
    field: &Field,
    ptr_offset: Option<TokenStream>,
) -> TokenStream {
    let field_ident = &field.ident;
    let ident = match &ptr_offset {
        Some(ptr_offset) => quote! {
            self.#field_ident.add(#ptr_offset)
        },
        None => quote! {
            self.#field_ident
        },
    };

    match &field.ty {
        FieldType::Reference => {
            quote! {
                ::cdump::CSerialize::serialize(&*#ident, buf);
            }
        }
        FieldType::CString => {
            *start_index = true;

            let set_len = match ptr_offset.is_none() {
                true => quote! {
                    ::cdump::internal::set_length_in_ptr(buf, start_index + ::cdump::offset_of!(Self, #field_ident), len);
                },
                false => quote! {},
            };

            quote! {
                let len = ::cdump::internal::libc_strlen(#ident) + 1;
                #set_len
                buf.push_slice(std::slice::from_raw_parts(#ident as *const _ as *const u8, len));
            }
        }
        FieldType::Array(len, inner) => {
            let inner_path = &inner.path;
            let inner = match inner.ty {
                FieldType::Reference => {
                    let ident = &inner.ident;
                    quote! {
                        ::cdump::CSerialize::serialize_without_shallow_copy(&*self.#ident.add(i), buf, array_start_index + size * i);
                    }
                }
                _ => todo!("support 2D arrays"),
            };

            quote! {
                let len = self.#len as usize;
                let size = ::std::mem::size_of::<#inner_path>();

                buf.align::<#inner_path>();
                let array_start_index = buf.len();
                buf.push_slice(std::slice::from_raw_parts(#ident as *const _ as *const u8, size * len));

                for i in 0..len {
                    #inner
                }
            }
        }
    }
}

#[proc_macro_derive(CDeserialize, attributes(cdump))]
pub fn c_deserialize_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let validate_repr = validate_repr(&ast.attrs, "C", ast.span()).to_compile_error();
    let name = ast.ident.clone();

    let align_and_read_copy = align_and_read_copy();
    let deep_fields = match field_analysis::get_fields(&ast) {
        Ok(fields) => read_deep_fields(fields),
        Err(err) => err.to_compile_error(),
    };

    proc_macro::TokenStream::from(quote! {
        impl<T: ::cdump::CDumpReader> ::cdump::CDeserialize<T> for #name {
            unsafe fn deserialize_to(buf: &mut T, dst: &mut Self) {
                #validate_repr

                #align_and_read_copy
                Self::deserialize_to_without_shallow_copy(buf, dst);
            }

            unsafe fn deserialize_to_without_shallow_copy(buf: &mut T, dst: &mut Self) {
                #deep_fields
            }
        }
    })
}

fn align_and_read_copy() -> TokenStream {
    quote! {
        buf.align::<Self>();
        let size = ::std::mem::size_of::<Self>();
        std::ptr::copy_nonoverlapping(buf.read_slice(size).as_ptr(), dst as *mut _ as *mut u8, size);
    }
}

fn read_deep_fields(fields: Vec<Field>) -> TokenStream {
    let mut quotes = Vec::new();

    for field in fields {
        quotes.push(read_deep_fields_inner(&field, 0));
    }

    quotes.into_iter().collect()
}

fn read_deep_fields_inner(field: &Field, ptr_offset: usize) -> TokenStream {
    let field_ident = &field.ident;
    let ident = match ptr_offset == 0 {
        true => quote! {
            dst.#field_ident
        },
        false => quote! {
            dst.#field_ident.add(#ptr_offset)
        },
    };
    let path = &field.path;

    match &field.ty {
        FieldType::Reference => {
            quote! {
                #ident = ::cdump::internal::deserialize_shallow_copied(buf);
            }
        }
        FieldType::CString => {
            quote! {
                #ident = buf.read_slice(#ident as usize).as_ptr() as *const ::std::ffi::c_char;
            }
        }
        FieldType::Array(len, inner) => {
            let inner_path = &inner.path;
            let (prefix, _start, inner) = match inner.ty {
                FieldType::Reference => (
                    {
                        quote! {
                            let len = dst.#len as usize;
                            let size = ::std::mem::size_of::<#inner_path>();

                            buf.align::<#inner_path>();
                            let array_start_index = buf.get_read();
                            buf.add_read(size * len);
                            #ident = ::cdump::internal::deserialize_shallow_copied_at(buf, array_start_index);
                        }
                    },
                    1usize,
                    quote! {
                        _ = ::cdump::internal::deserialize_shallow_copied_at::<T, #path>(buf, array_start_index + size * i);
                    },
                ),
                FieldType::CString => (quote! {}, 1usize, quote! {}),
                _ => (
                    quote! {},
                    0usize,
                    read_deep_fields_inner(inner, ptr_offset + 1),
                ),
            };

            quote! {
                #prefix
                for i in 1..len {
                    #inner
                }
            }
        }
    }
}

#[proc_macro_attribute]
pub fn dynamic_serializator(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}
