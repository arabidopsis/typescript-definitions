extern crate proc_macro2;
extern crate syn;

use super::{ast, type_to_ts, QuoteT};
use proc_macro2::{Ident, Span};

pub fn ident_from_str(s: &str) -> Ident {
    syn::Ident::new(s, Span::call_site())
}

pub fn derive_field<'a>(field: &ast::Field<'a>) -> QuoteT {
    let field_name = field.attrs.name().serialize_name(); // use serde name instead of field.member
    let field_name = ident_from_str(&field_name);

    let ty = type_to_ts(&field.ty);

    quote! {
        #field_name: #ty
    }
}

pub fn field_type_name(ty: &syn::Type) -> Option<String> {
    use syn::Type::Path;
    match ty {
        Path(syn::TypePath { path, .. }) => match path.segments.last().map(|p| p.into_value()) {
            Some(t) => Some(t.ident.to_string()),
            _ => None,
        },
        _ => None,
    }
}
pub fn full_field_type_name(ty: &syn::Type) -> Option<Vec<Ident>> {
    use syn::Type::Path;
    match ty {
        Path(syn::TypePath { path, .. }) => {
            Some(path.segments.iter().map(|p| p.ident.clone()).collect())
        }
        _ => None,
    }
}

pub fn is_phantom(ty: &syn::Type) -> bool {
    match field_type_name(ty) {
        Some(t) => t == "PhantomData",
        _ => false,
    }
}

pub fn filter_visible<'a>(fields: &'a [ast::Field<'a>]) -> Vec<&'a ast::Field<'a>> {
    let mut content: Vec<&'a ast::Field<'a>> = Vec::with_capacity(fields.len());

    for field in fields {
        if field.attrs.skip_serializing() || is_phantom(field.ty) {
            continue;
        }

        content.push(field);
    }
    content
}
