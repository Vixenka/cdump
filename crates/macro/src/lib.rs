use field_analysis::{Field, FieldType};
use helpers::{is_primitive_type, validate_repr, ErrorExt};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident, TypePath};

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
            unsafe fn serialize(&self, buf: &mut T) {
                #validate_repr

                ::cdump::internal::align_writer::<T, Self>(buf);
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
        FieldType::Dynamic(serializer, _) => quote! {
            #serializer(buf, #ident);
        },
        FieldType::Array(len, inner) => {
            let mut inner_path = inner.path.to_token_stream();
            if let FieldType::CString = inner.ty {
                inner_path = quote! { *const #inner_path };
            }

            let mut result = quote! {
                let len = (#len) as usize;
                let size = ::std::mem::size_of::<#inner_path>();

                ::cdump::internal::align_writer::<T, #inner_path>(buf);
                let array_start_index = buf.len();
                buf.push_slice(std::slice::from_raw_parts(#ident as *const _ as *const u8, size * len));
            };

            if !is_primitive_type(&inner_path) {
                let inner = get_inner_of_array_serialize(inner, &ident);
                result = quote! {
                    #result

                    for i in 0..len {
                        #inner
                    }
                };
            }

            result
        }
    }
}

fn get_inner_of_array_serialize(inner: &Field, ident: &TokenStream) -> TokenStream {
    match inner.ty {
        FieldType::Reference => {
            let ident = &inner.ident;
            quote! {
                ::cdump::CSerialize::serialize_without_shallow_copy(&*self.#ident.add(i), buf, array_start_index + size * i);
            }
        }
        FieldType::CString => {
            quote! {
                let ptr = *#ident.add(i);
                let len = ::cdump::internal::libc_strlen(ptr) + 1;
                ::cdump::internal::set_length_in_ptr(buf, array_start_index + size * i, len);
                buf.push_slice(std::slice::from_raw_parts(ptr as *const _ as *const u8, len));
            }
        }
        _ => unimplemented!("2D arrays"),
    }
}

#[proc_macro_derive(CDeserialize, attributes(cdump))]
pub fn c_deserialize_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let validate_repr = validate_repr(&ast.attrs, "C", ast.span()).to_compile_error();
    let name = ast.ident.clone();

    let align_and_read_copy = align_and_read_copy();
    let deep_fields = match field_analysis::get_fields(&ast) {
        Ok(fields) => read_deep_fields(fields, &name),
        Err(err) => err.to_compile_error(),
    };

    proc_macro::TokenStream::from(quote! {
        impl<T: ::cdump::CDumpReader> ::cdump::CDeserialize<T> for #name {
            unsafe fn deserialize_to(buf: &mut T, dst: *mut Self) {
                #validate_repr

                #align_and_read_copy
                Self::deserialize_to_without_shallow_copy(buf, dst);
            }

            unsafe fn deserialize_to_without_shallow_copy(buf: &mut T, dst: *mut Self) {
                #deep_fields
            }
        }
    })
}

fn align_and_read_copy() -> TokenStream {
    quote! {
        ::cdump::internal::align_reader::<T, Self>(buf);
        let size = ::std::mem::size_of::<Self>();
        std::ptr::copy_nonoverlapping(buf.read_raw_slice(size), dst as *mut _ as *mut u8, size);
    }
}

fn read_deep_fields(fields: Vec<Field>, name: &proc_macro2::Ident) -> TokenStream {
    let mut quotes = Vec::new();

    for (index, field) in fields.iter().enumerate() {
        quotes.push(read_deep_fields_inner(field, 0, name, index));
    }

    quotes.into_iter().collect()
}

fn read_deep_fields_inner(
    field: &Field,
    ptr_offset: usize,
    name: &proc_macro2::Ident,
    field_index: usize,
) -> TokenStream {
    let field_ident = &field.ident;
    let ident = match ptr_offset == 0 {
        true => quote! {
            (*dst).#field_ident
        },
        false => quote! {
            (*dst).#field_ident.add(#ptr_offset)
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
                #ident = buf.read_raw_slice(#ident as usize) as *const ::std::ffi::c_char;
            }
        }
        FieldType::Dynamic(_, deserializer) => quote! {
            #ident = #deserializer(buf);
        },
        FieldType::Array(len, inner) => {
            let mut inner_path = inner.path.to_token_stream();
            if let FieldType::CString = inner.ty {
                inner_path = quote! { *const #inner_path };
            }

            let len_function = Ident::new(
                &format!(
                    "do_not_use_cdump_internal_function_len_of_array_at_index_{}",
                    field_index
                ),
                Span::call_site(),
            );

            let mut result = quote! {
                impl #name {
                    #[inline]
                    #[doc(hidden)]
                    fn #len_function(&self) -> usize {
                        (#len) as usize
                    }
                }

                let len = (*dst).#len_function();
                let size = ::std::mem::size_of::<#inner_path>();

                ::cdump::internal::align_reader::<T, #inner_path>(buf);
            };

            if !is_primitive_type(&inner_path) {
                let (prefix, start, inner) = get_inner_of_array_deserialize(inner, &ident, path);
                result = quote! {
                    #result
                    let array_start_index = buf.get_read();

                    #prefix
                    for i in #start..len {
                        #inner
                    }
                };
            } else {
                result = quote! {
                    #result
                    #ident = buf.read_raw_slice(size * len) as *const #inner_path;
                };
            }

            result
        }
    }
}

fn get_inner_of_array_deserialize(
    inner: &Field,
    ident: &TokenStream,
    path: &Option<TypePath>,
) -> (TokenStream, TokenStream, TokenStream) {
    match inner.ty {
        FieldType::Reference => (
            quote! {
                buf.add_read(size * len);
                #ident = ::cdump::internal::deserialize_shallow_copied_at(buf, array_start_index);
            },
            quote! { 1 },
            quote! {
                _ = ::cdump::internal::deserialize_shallow_copied_at::<T, #path>(buf, array_start_index + size * i);
            },
        ),
        FieldType::CString => (
            quote! {
                #ident = buf.read_raw_slice(size * len) as *const *const ::std::ffi::c_char;
            },
            quote! { 0 },
            quote! {
                let ptr = buf.as_mut_ptr_at(array_start_index + size * i);
                *ptr = buf.read_raw_slice(*ptr as usize) as *const ::std::ffi::c_char;
            },
        ),
        _ => unimplemented!("2D arrays"),
    }
}
