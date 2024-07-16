use darling::{ast::Data, FromDeriveInput};
use syn::{spanned::Spanned, DeriveInput, Error, Ident, Type};

pub struct Field {
    pub ident: Option<Ident>,
    pub ty: FieldType,
}

pub enum FieldType {
    CString,
}

pub fn get_fields(ast: &DeriveInput) -> Result<Vec<Field>, Error> {
    let receiver = InputReceiver::from_derive_input(ast).unwrap();
    let mut vec = Vec::new();

    for field in &receiver.data.take_struct().unwrap().fields {
        if let Type::Ptr(_) = &field.ty {
            let (ty, _ptr_level) = extract_ptr(&field.ty);
            let Type::Path(path) = ty else {
                return Err(Error::new(
                    field.ty.span(),
                    "cannot extract path from field",
                ));
            };

            if !path.path.is_ident("c_char") {
                continue;
            }

            vec.push(Field {
                ident: field.ident.clone(),
                ty: FieldType::CString,
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

#[derive(darling::FromDeriveInput)]
struct InputReceiver {
    data: Data<(), FieldReceiver>,
}

#[derive(darling::FromField)]
struct FieldReceiver {
    ident: Option<Ident>,
    ty: Type,
}
