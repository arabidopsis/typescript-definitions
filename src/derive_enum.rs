// use quote::TokenStreamExt;
use serde_derive_internals::{ast, attr, attr::EnumTag};

use super::{
    derive_element, derive_field, type_to_ts, QuoteT,
};

struct TagInfo<'a> {
    tag: &'a str,
    content: Option<&'a str>,
}
pub(crate) fn derive_enum<'a>(variants: &[ast::Variant<'a>], attrs: &attr::Container) -> QuoteT {
    // let n = variants.len() - 1;
    let taginfo = match attrs.tag() {
        EnumTag::Internal { tag, .. } => TagInfo { tag, content: None },
        EnumTag::Adjacent { tag, content, .. } => TagInfo {
            tag,
            content: Some(&content),
        },
        _ => TagInfo {
            tag: "kind",
            content: None,
        },
    };
    let content = variants
        .into_iter()
        .map(|variant| {
            let variant_name = variant.attrs.name().serialize_name();
            match variant.style {
                ast::Style::Struct => {
                    derive_struct_variant(&taginfo, &variant_name, &variant.fields)
                }
                ast::Style::Newtype => {
                    derive_newtype_variant(&taginfo, &variant_name, &variant.fields[0])
                }
                ast::Style::Tuple => derive_tuple_variant(&taginfo, &variant_name, &variant.fields),
                ast::Style::Unit => derive_unit_variant(&taginfo, &variant_name),
            }
        });

        // .enumerate()
        // .fold(quote! {}, |mut agg, (i, tokens)| {
        //     agg.append_all(tokens);
        //     if i < n {
        //         agg.append_all(quote! {|})
        //     }
        //     agg
        // })
        quote!{ #(#content)|* }
}

fn derive_unit_variant(taginfo: &TagInfo, variant_name: &str) -> QuoteT {
    let tag = taginfo.tag;
    quote! {
        { #tag: #variant_name }
    }
}

fn derive_newtype_variant<'a>(
    taginfo: &TagInfo,
    variant_name: &str,
    field: &ast::Field<'a>,
) -> QuoteT {
    let ty = type_to_ts(&field.ty);
    let tag = taginfo.tag;
    if let Some(content) = taginfo.content {
        quote! {
         { #tag: #variant_name, #content: #ty }
        }
    } else {
        quote! {
         { #tag: #variant_name, "fields": #ty }
        }
    }
}

fn derive_struct_variant<'a>(
    taginfo: &TagInfo,
    variant_name: &str,
    fields: &[ast::Field<'a>],
) -> QuoteT {
    let contents = fields
        .iter()
        .map(|field| derive_field(field));
        // .collect::<Vec<_>>();

    let tag = taginfo.tag;
    if let Some(content) = taginfo.content {
        // let contents = collapse_list_brace(&contents);
        quote! {
            { #tag: #variant_name, #content: { #(#contents),* } }
        }
    } else {
        quote! {
            { #tag: #variant_name, #(#contents),* }
        }
    }
}

fn derive_tuple_variant<'a>(
    taginfo: &TagInfo,
    variant_name: &str,
    fields: &[ast::Field<'a>],
) -> QuoteT {
    let contents =
        fields
            .iter()
            .map(|field| derive_element(field));
            // .collect::<Vec<_>>();
    
    let tag = taginfo.tag;
    if let Some(content) = taginfo.content {
        quote! {
         { #tag: #variant_name, #content: [ #(#contents),* ] }
        }
    } else {
        quote! {
         { #tag: #variant_name, "fields": [ #(#contents),* ] }
        }
    }
}
