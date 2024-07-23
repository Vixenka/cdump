use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{AttrStyle, Attribute, Error, Path};

pub fn is_primitive_type(path: &TokenStream) -> bool {
    let path = path.to_string();
    path == "u8"
        || path == "u16"
        || path == "u32"
        || path == "u64"
        || path == "u128"
        || path == "usize"
        || path == "i8"
        || path == "i16"
        || path == "i32"
        || path == "i64"
        || path == "i128"
        || path == "isize"
        || path == "f32"
        || path == "f64"
        || path == "bool"
}

pub fn validate_repr(attrs: &[Attribute], repr: &str, span: Span) -> Result<(), Error> {
    let mut index = None;

    for (i, attr) in attrs.iter().enumerate() {
        if !matches!(attr.style, AttrStyle::Outer) {
            continue;
        }

        if let Path {
            leading_colon: None,
            ref segments,
        } = attr.path()
        {
            if segments.len() != 1 {
                continue;
            }

            let seg = segments.first().unwrap();
            if !seg.arguments.is_empty() {
                continue;
            }

            if seg.ident != "repr" {
                continue;
            } else {
                index = Some(i);
            }
        } else {
            continue;
        }

        let mut attr = format!("{}", attr.to_token_stream());
        attr = attr.replace(' ', "");
        if attr != format!("#[repr({})]", repr) {
            continue;
        }

        return Ok(());
    }

    Err(Error::new(
        match index {
            Some(index) => attrs[index].span(),
            _ => span,
        },
        format!("expected `#[repr({})]`", repr),
    ))
}

pub trait ErrorExt {
    fn to_compile_error(&self) -> proc_macro2::TokenStream;
}

impl<T> ErrorExt for Result<T, Error> {
    fn to_compile_error(&self) -> proc_macro2::TokenStream {
        match self {
            Ok(_) => proc_macro2::TokenStream::new(),
            Err(e) => e.to_compile_error(),
        }
    }
}
