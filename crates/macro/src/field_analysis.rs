use darling::{ast::Data, FromDeriveInput};
use syn::{spanned::Spanned, DeriveInput, Error, Ident, Type, TypePath};

pub struct Field {
    pub ident: Option<Ident>,
    pub path: Option<TypePath>,
    pub ty: FieldType,
}

pub enum FieldType {
    Reference,
    CString,
    Array(Ident, Box<Field>),
}

pub fn get_fields(ast: &DeriveInput) -> Result<Vec<Field>, Error> {
    let receiver = InputReceiver::from_derive_input(ast).unwrap();
    let mut vec = Vec::new();

    for field in &receiver.data.take_struct().unwrap().fields {
        if let Type::Ptr(_) = &field.ty {
            let (ty, ptr_level) = extract_ptr(&field.ty);
            let is_c_char = is_c_char(ty);

            let path = match ty {
                Type::Path(path) => Some(path.clone()),
                _ => None,
            };

            if ptr_level != 1 {
                if is_c_char {
                    if field.array.is_none() {
                        return Err(Error::new(
                            field.ty.span(),
                            "two levels of pointer in CString, requires field to be an array",
                        ));
                    }

                    if ptr_level > 2 {
                        return Err(Error::new(
                            field.ty.span(),
                            "more than two levels of pointer in CString is not supported",
                        ));
                    }
                } else {
                    return Err(Error::new(
                        field.ty.span(),
                        "only one level of pointer is supported for a reference",
                    ));
                }
            }

            let fty = match is_c_char {
                true => FieldType::CString,
                false => FieldType::Reference,
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
                            ty: fty,
                        }),
                    ),
                    None => fty,
                },
            });
        }
    }

    Ok(vec)
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

fn is_c_char(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path.path.is_ident("c_char"),
        _ => false,
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
}

#[derive(darling::FromMeta)]
struct ArrayReceiver {
    len: Ident,
}
