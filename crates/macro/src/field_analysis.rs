use darling::{ast::Data, FromDeriveInput};
use syn::{spanned::Spanned, DeriveInput, Error, Expr, Ident, Type, TypePath};

pub struct Field {
    pub ident: Option<Ident>,
    pub path: Option<TypePath>,
    pub ty: FieldType,
}

pub enum FieldType {
    Plain,
    Reference,
    CString,
    Array(Expr, Box<Field>),
    Dynamic(Ident, Ident),
}

pub fn get_fields(ast: &DeriveInput) -> Result<Vec<Field>, Error> {
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
                    FieldType::Dynamic(dynamic.serializer.clone(), dynamic.deserializer.clone())
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

    if raw_ty == RawFieldType::Dynamic && field.dynamic.is_none() {
        return Err(Error::new(
            field.ty.span(),
            "dynamic field requires provide a serializer and deserializer",
        ));
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RawFieldType {
    Reference,
    CString,
    Dynamic,
}
