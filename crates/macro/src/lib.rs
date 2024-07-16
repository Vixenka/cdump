use field_analysis::{Field, FieldType};
use helpers::{validate_repr, ErrorExt};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

mod field_analysis;
mod helpers;

#[proc_macro_derive(CSerialize)]
pub fn c_serialize_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let validate_repr = validate_repr(&ast.attrs, "C", ast.span()).to_compile_error();

    let name = ast.ident.clone();
    let align_and_push_copy = align_and_push_copy();

    let (start_index, get_fields) = match field_analysis::get_fields(&ast) {
        Ok(fields) => write_deep_fields(fields),
        Err(err) => (quote! {}, err.to_compile_error()),
    };

    proc_macro::TokenStream::from(quote! {
        impl<T: ::cdump::CDumpWriter> ::cdump::CSerialize<T> for #name {
            fn serialize(&self, buf: &mut T) {
                #validate_repr

                #start_index
                #align_and_push_copy
                #get_fields
            }
        }
    })
}

fn align_and_push_copy() -> TokenStream {
    quote! {
        buf.align::<Self>();
        buf.push_slice(unsafe {
            std::slice::from_raw_parts(self as *const _ as *const u8, std::mem::size_of::<Self>())
        });
    }
}

fn write_deep_fields(fields: Vec<Field>) -> (TokenStream, TokenStream) {
    let mut start_index = false;
    let mut quotes = Vec::new();

    for field in fields {
        let ident = &field.ident;

        quotes.push(match field.ty {
            FieldType::Reference => {
                quote! {
                    unsafe {
                        ::cdump::CSerialize::serialize(&*self.#ident, buf);
                    }
                }
            },
            FieldType::CString => {
                start_index = true;
                quote! {
                    unsafe {
                        let len = ::cdump::internal::libc_strlen(self.#ident);
                        ::cdump::internal::set_length_in_ptr(buf, start_index + ::cdump::offset_of!(Self, #ident), len);
                        buf.push_slice(std::slice::from_raw_parts(self.#ident as *const _ as *const u8, len + 1));
                    }
                }
            }
        });
    }

    (
        match start_index {
            true => quote! {
                let start_index = buf.len();
            },
            false => quote! {},
        },
        quotes.into_iter().collect(),
    )
}

#[proc_macro_derive(CDeserialize)]
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
        let ident = &field.ident;

        quotes.push(match field.ty {
            FieldType::Reference => {
                quote! {
                    dst.#ident = ::cdump::internal::deserialize_shallow_copied(buf);
                }
            },
            FieldType::CString => {
                quote! {
                    dst.#ident = buf.read_slice(dst.#ident as usize).as_ptr() as *const ::std::ffi::c_char;
                }
            }
        });
    }

    quotes.into_iter().collect()
}

#[proc_macro_attribute]
pub fn dynamic_serializator(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}
