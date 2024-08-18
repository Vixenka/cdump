use darling::{ast::Data, FromDeriveInput};
use syn::{spanned::Spanned, DeriveInput, Error, Expr, Ident, Type, TypeArray, TypePath};

pub struct Field {
    pub ident: Option<Ident>,
    pub path: Option<TypePath>,
    pub ty: FieldType,
}

pub enum FieldType {
    Plain,
    InlineArray(TypeArray),
    Reference,
    CString,
    Array(Expr, Box<Field>),
    Dynamic(DynamicField),
}

pub struct DynamicField {
    pub serializer: Ident,
    pub deserializer: Ident,
    pub ptr_level: usize,
    #[cfg(feature = "cdebug")]
    pub cdebugger: Option<Ident>,
}

#[cfg(feature = "cdebug")]
impl DynamicField {
    pub fn call_cdebugger(&self, ptr: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match &self.cdebugger {
            Some(cdebugger) => quote::quote! { unsafe { #cdebugger(#ptr) } },
            None => quote::quote! { &"<missing cdebugger>" },
        }
    }
}

pub fn get_fields(ast: &DeriveInput, skip_shallow_part: bool) -> Result<Vec<Field>, Error> {
    let receiver = InputReceiver::from_derive_input(ast).unwrap();
    let mut vec = Vec::new();

    for field in &receiver.data.take_struct().unwrap().fields {
        if let Type::Ptr(_) = &field.ty {
            let (ty, ptr_level) = extract_ptr(&field.ty);
            let raw_ty = get_raw_field_type(ty);

            let path = match ty {
                Type::Path(path) => Some(path.clone()),
                _ => None,
            };

            validate_field(raw_ty, ptr_level, field)?;

            let fty = match raw_ty {
                RawFieldType::Reference => FieldType::Reference,
                RawFieldType::CString => FieldType::CString,
                RawFieldType::Dynamic => {
                    let dynamic = field.dynamic.as_ref().unwrap();
                    FieldType::Dynamic(DynamicField {
                        serializer: dynamic.serializer.clone(),
                        deserializer: dynamic.deserializer.clone(),
                        ptr_level,
                        #[cfg(feature = "cdebug")]
                        cdebugger: dynamic.cdebugger.clone(),
                    })
                }
            };

            vec.push(Field {
                ident: field.ident.clone(),
                path: path.clone(),
                ty: match &field.array {
                    Some(array) => FieldType::Array(
                        array.len.clone(),
                        Box::new(Field {
                            ident: field.ident.clone(),
                            path,
                            ty: match fty {
                                FieldType::Reference => match ptr_level == 1 {
                                    true => FieldType::Plain,
                                    false => FieldType::Reference,
                                },
                                _ => fty,
                            },
                        }),
                    ),
                    None => fty,
                },
            });
        } else if !skip_shallow_part {
            vec.push(Field {
                ident: field.ident.clone(),
                path: match &field.ty {
                    Type::Path(path) => Some(path.clone()),
                    _ => None,
                },
                ty: match &field.ty {
                    Type::Array(array) => FieldType::InlineArray(array.clone()),
                    _ => FieldType::Plain,
                },
            })
        }
    }

    Ok(vec)
}

fn validate_field(
    raw_ty: RawFieldType,
    ptr_level: usize,
    field: &FieldReceiver,
) -> Result<(), Error> {
    if ptr_level != 1 {
        if field.array.is_none() {
            return Err(Error::new(
                field.ty.span(),
                "two levels of pointer, requires field to be an array",
            ));
        }

        if ptr_level > 2 {
            return Err(Error::new(
                field.ty.span(),
                "more than two levels of pointer is not supported",
            ));
        }
    }

    if raw_ty == RawFieldType::Dynamic {
        if field.dynamic.is_none() {
            return Err(Error::new(
                field.ty.span(),
                "dynamic field requires provide a serializer and deserializer",
            ));
        } else if ptr_level == 1 && field.array.is_some() {
            return Err(Error::new(
                field.ty.span(),
                "array of dynamic field under one level of pointer is not supported",
            ));
        }
    }

    Ok(())
}

fn extract_ptr(ty: &Type) -> (&Type, usize) {
    match ty {
        Type::Ptr(ty) => {
            let (inner_ty, ptr_level) = extract_ptr(&ty.elem);
            (inner_ty, ptr_level + 1)
        }
        _ => (ty, 0),
    }
}

fn get_raw_field_type(ty: &Type) -> RawFieldType {
    match ty {
        Type::Path(path) => {
            if path.path.is_ident("c_char") || path.path.is_ident("std::ffi::c_char") {
                return RawFieldType::CString;
            } else if path.path.is_ident("c_void") || path.path.is_ident("std::ffi::c_void") {
                return RawFieldType::Dynamic;
            }
            RawFieldType::Reference
        }
        _ => RawFieldType::Reference,
    }
}

#[derive(darling::FromDeriveInput)]
struct InputReceiver {
    data: Data<(), FieldReceiver>,
}

#[derive(darling::FromField)]
#[darling(attributes(cdump))]
struct FieldReceiver {
    ident: Option<Ident>,
    ty: Type,
    array: Option<ArrayReceiver>,
    dynamic: Option<DynamicReceiver>,
}

#[derive(darling::FromMeta)]
struct ArrayReceiver {
    len: Expr,
}

#[derive(darling::FromMeta)]
struct DynamicReceiver {
    serializer: Ident,
    deserializer: Ident,
    #[cfg(feature = "cdebug")]
    cdebugger: Option<Ident>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RawFieldType {
    Reference,
    CString,
    Dynamic,
}
