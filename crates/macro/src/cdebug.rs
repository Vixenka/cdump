use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::field_analysis::{self, Field, FieldType};

pub fn c_debug_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.clone();

    let write = match field_analysis::get_fields(&ast, false) {
        Ok(fields) => write_fmt(&fields),
        Err(err) => err.to_compile_error(),
    };

    let name_str = name.to_string();
    proc_macro::TokenStream::from(quote! {
        impl ::std::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(#name_str)
                    #write
                    .finish()
            }
        }
    })
}

fn write_fmt(fields: &[Field]) -> proc_macro2::TokenStream {
    let mut quotes = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref();
        let ident_str = ident.expect("expected field to have ident").to_string();

        let value = match &field.ty {
            FieldType::Plain => quote! {
                &self.#ident
            },
            FieldType::Reference => quote! {
                &unsafe { self.#ident.as_ref() }
            },
            FieldType::CString => quote! {
                &unsafe { ::std::ffi::CStr::from_ptr(self.#ident) }
            },
            FieldType::Array(len, ty) => {
                let extension = match ty.ty {
                    FieldType::Reference => quote! {
                        .iter().map(|&ptr| ptr.as_ref()).collect::<Vec<_>>()
                    },
                    FieldType::CString => quote! {
                        .iter().map(|&ptr| ::std::ffi::CStr::from_ptr(ptr)).collect::<Vec<_>>()
                    },
                    _ => quote! {},
                };

                quote! {
                    &match self.#ident.is_null() {
                        true => None,
                        false => Some(unsafe { ::std::slice::from_raw_parts(self.#ident, (#len) as usize)#extension }),
                    }
                }
            }
            FieldType::Dynamic(_, _, _) => todo!(),
        };

        quotes.push(quote! {
            .field(#ident_str, #value)
        });
    }

    quotes.into_iter().collect()
}
