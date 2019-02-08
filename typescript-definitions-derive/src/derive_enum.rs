// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde_derive_internals::{ast, attr::EnumTag, Ctxt};

use super::{derive_field, ident_from_str, type_to_ts, QuoteT, filter_visible};

const CONTENT: &'static str = "fields"; // default content tag

struct TagInfo<'a> {
    tag: &'a str,
    content: Option<&'a str>,
}
pub(crate) fn derive_enum<'a>(
    variants: &[ast::Variant<'a>],
    container: &ast::Container,
    cx: &Ctxt,
) -> (bool /* is enum */, QuoteT) {
    let taginfo = match container.attrs.tag() {
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
    // check for #[serde(skip)]
    let mut skip_variants : Vec<&ast::Variant<'a>> = Vec::with_capacity(variants.len());
    for v in variants {
        if v.attrs.skip_serializing() { continue; }
        skip_variants.push(v); 
    }


    let mut is_enum = true;
    for v in &skip_variants {
        match v.style {
            ast::Style::Unit => continue,
            _ => {
                is_enum = false;
                break;
            }
        }
    }
    if is_enum {
        let v = &skip_variants
            .iter()
            .map(|v| v.attrs.name().serialize_name()) // use serde name instead of v.ident
            .collect::<Vec<_>>();
        let k = v.iter().map(|v| ident_from_str(&v)).collect::<Vec<_>>();
        return (true, quote! ( { #(#k = #v),* } ));
    }



    let content = skip_variants.iter().map(|variant| {
        let variant_name = variant.attrs.name().serialize_name(); // use serde name instead of variant.ident
        match variant.style {
            ast::Style::Struct => {
                derive_struct_variant(&taginfo, &variant_name, &variant.fields, container, cx)
            }
            ast::Style::Newtype => {
                derive_newtype_variant(&taginfo, &variant_name, &variant.fields[0])
            }
            ast::Style::Tuple => derive_tuple_variant(&taginfo, &variant_name, &variant.fields),
            ast::Style::Unit => derive_unit_variant(&taginfo, &variant_name),
        }
    });
    // OK generate A | B | C etc
    (false, quote! { #(#content)|* })
}

fn derive_unit_variant(taginfo: &TagInfo, variant_name: &str) -> QuoteT {
    let tag = ident_from_str(taginfo.tag);
    quote! {
        { #tag: #variant_name }
    }
}

fn derive_newtype_variant<'a>(
    taginfo: &TagInfo,
    variant_name: &str,
    field: &ast::Field<'a>,
) -> QuoteT {
    if field.attrs.skip_serializing() {
        return derive_unit_variant(taginfo, variant_name);
    }
    let ty = type_to_ts(&field.ty);
    let tag = ident_from_str(taginfo.tag);
    let content = if let Some(content) = taginfo.content {
        ident_from_str(&content)
    } else {
        ident_from_str(CONTENT)
    };

    quote! {
        { #tag: #variant_name, #content: #ty }
    }
}

fn derive_struct_variant<'a>(
    taginfo: &TagInfo,
    variant_name: &str,
    fields: &[ast::Field<'a>],
    container: &ast::Container,
    cx: &Ctxt, // for error reporting
) -> QuoteT {
    use std::collections::HashSet;
    let fields = filter_visible(fields);
    if fields.len() == 0 {
        return derive_unit_variant(taginfo, variant_name);
    }


    let contents = fields.iter().map(|f| derive_field(f));


    let tag = ident_from_str(taginfo.tag);
    if let Some(content) = taginfo.content {
        let content = ident_from_str(&content);
        quote! {
            { #tag: #variant_name, #content: { #(#contents),* } }
        }
    } else {
        let fnames = fields
            .iter()
            .map(|field| field.attrs.name().serialize_name())
            .collect::<HashSet<_>>();
        if fnames.contains(taginfo.tag) {
            cx.error(format!(
                "tag \"{}\" clashes with field in Enum variant \"{}::{}\". \
                 Maybe use a #[serde(content=\"...\")] attribute.",
                taginfo.tag, container.ident, variant_name
            ));
        }
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
    let fields = filter_visible(fields);
    let contents = fields.iter().map(|field| type_to_ts(&field.ty));

    let tag = ident_from_str(taginfo.tag);
    let content = if let Some(content) = taginfo.content {
        ident_from_str(&content)
    } else {
        ident_from_str(CONTENT)
    };

    quote! {
     { #tag: #variant_name, #content : [ #(#contents),* ] }
    }
}
