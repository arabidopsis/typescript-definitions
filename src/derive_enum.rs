
use quote::TokenStreamExt;
use serde_derive_internals::{ast, attr};

use super::{derive_element, derive_field, QuoteT,collapse_list_brace, collapse_list_bracket, type_to_ts};

pub fn derive_enum<'a>(
    variants: Vec<ast::Variant<'a>>,
    _attr_container: &attr::Container,
) -> QuoteT {
    variants
        .into_iter()
        .enumerate()
        .map(|(variant_idx, variant)| {
            let variant_name = variant.attrs.name().serialize_name();
            match variant.style {
                ast::Style::Struct => {
                    derive_struct_variant(&variant_name, variant_idx, &variant.fields)
                }
                ast::Style::Newtype => {
                    derive_newtype_variant(&variant_name, variant_idx, &variant.fields[0])
                }
                ast::Style::Tuple => {
                    derive_tuple_variant(&variant_name, variant_idx, &variant.fields)
                }
                ast::Style::Unit => derive_unit_variant(&variant_name),
            }
        })
        .fold(quote! {}, |mut agg, tokens| {
            agg.append_all(tokens);
            agg
        })
}

fn derive_unit_variant(variant_name: &str) -> QuoteT {
    quote! {
        | { "tag": #variant_name }
    }
}

fn derive_newtype_variant<'a>(
    variant_name: &str,
    _variant_idx: usize,
    field: &ast::Field<'a>,
) -> QuoteT {
    let ty = type_to_ts(&field.ty);
    quote! {
        | { "tag": #variant_name, "fields": #ty }
    }
}

fn derive_struct_variant<'a>(
    variant_name: &str,
    variant_idx: usize,
    fields: &[ast::Field<'a>],
) -> QuoteT {
    let contents = collapse_list_brace(
        &fields
            .into_iter()
            .enumerate()
            .map(|(field_idx, field)| derive_field(variant_idx, field_idx, field))
            .collect::<Vec<_>>()
    );
    quote! {
        | { "tag": #variant_name, "fields": #contents }
    }
}

fn derive_tuple_variant<'a>(
    variant_name: &str,
    _variant_idx: usize,
    fields: &[ast::Field<'a>],
) -> QuoteT {
    let contents = collapse_list_bracket(
        &fields
            .into_iter()
            .enumerate()
            .map(|(element_idx, field)| derive_element(0, element_idx, &field))
            .collect::<Vec<_>>()
    );
    quote! {
        | { "tag": #variant_name, "fields": #contents }
    }
}
