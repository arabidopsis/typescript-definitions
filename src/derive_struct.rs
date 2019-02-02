use serde_derive_internals::{ast, attr};

use super::{
    collapse_list_brace, collapse_list_bracket, derive_element, derive_field, type_to_ts, QuoteT,
};

pub fn derive_struct<'a>(
    style: ast::Style,
    fields: &[ast::Field<'a>],
    attr_container: &attr::Container,
) -> QuoteT {
    match style {
        ast::Style::Struct => derive_struct_named_fields(fields, attr_container),
        ast::Style::Newtype => derive_struct_newtype(fields, attr_container),
        ast::Style::Tuple => derive_struct_tuple(fields, attr_container),
        ast::Style::Unit => derive_struct_unit(attr_container),
    }
}

fn derive_struct_newtype<'a>(
    fields: &[ast::Field<'a>],
    _attr_container: &attr::Container,
) -> QuoteT {
    derive_element(&fields[0])
}

fn derive_struct_unit(_attr_container: &attr::Container) -> QuoteT {
    quote! {
        {}
    }
}

fn derive_struct_named_fields<'a>(
    fields: &[ast::Field<'a>],
    _attr_container: &attr::Container,
) -> QuoteT {
    collapse_list_brace(
        &fields
            .into_iter()
            .map(|field| derive_field(&field))
            .collect::<Vec<_>>()
    )
}

fn derive_struct_tuple<'a>(fields: &[ast::Field<'a>], _attr_container: &attr::Container) -> QuoteT {
    collapse_list_bracket(
        &fields
            .into_iter()
            .map(|field| type_to_ts(field.ty))
            .collect::<Vec<_>>()
    )
}
