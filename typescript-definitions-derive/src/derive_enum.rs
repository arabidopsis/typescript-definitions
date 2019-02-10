// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde_derive_internals::{ast, attr::EnumTag, ast::Variant};

use super::{filter_visible, ident_from_str, ParseContext, QuoteMaker};

const CONTENT: &'static str = "fields"; // default content tag
const TAG: &'static str = "kind"; // default tag tag

struct TagInfo<'a> {
    tag: &'a str,
    content: Option<&'a str>,
}
impl<'a> ParseContext<'_> {
    pub(crate) fn derive_enum(
        &self,
        variants: &[ast::Variant<'a>],
        container: &ast::Container,
    ) -> QuoteMaker {
        let taginfo = match container.attrs.tag() {
            EnumTag::Internal { tag, .. } => TagInfo { tag, content: None },
            EnumTag::Adjacent { tag, content, .. } => TagInfo {
                tag,
                content: Some(&content),
            },
            _ => TagInfo {
                tag: TAG,
                content: None,
            },
        };
        // check for #[serde(skip)]
        let mut skip_variants: Vec<&ast::Variant<'a>> = Vec::with_capacity(variants.len());
        for v in variants {
            if v.attrs.skip_serializing() {
                continue;
            }
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
            self.is_enum.set(is_enum); // tell parser context
            let v = &skip_variants
                .iter()
                .map(|v| v.attrs.name().serialize_name()) // use serde name instead of v.ident
                .collect::<Vec<_>>();
            let k = v.iter().map(|v| ident_from_str(&v)).collect::<Vec<_>>();

            return quote! ( { #(#k = #v),* } ).into();
        }

        let content = skip_variants.iter().map(|variant| {
            match variant.style {
                ast::Style::Struct => {
                    self.derive_struct_variant(&taginfo, variant, &variant.fields, container)
                }
                ast::Style::Newtype => {
                    self.derive_newtype_variant(&taginfo, variant, &variant.fields[0])
                }
                ast::Style::Tuple => {
                    self.derive_tuple_variant(&taginfo, variant, &variant.fields)
                }
                ast::Style::Unit => self.derive_unit_variant(&taginfo, variant),
            }
        });
        // OK generate A | B | C etc
        quote! ( #(#content)|* ).into()
    }

    fn derive_unit_variant(&self, taginfo: &TagInfo, variant: &Variant) -> QuoteMaker {
        let tag = ident_from_str(taginfo.tag);
        let variant_name = variant.attrs.name().serialize_name(); // use serde name instead of variant.ident
        quote! (
            { #tag: #variant_name }
        )
        .into()
    }

    fn derive_newtype_variant(
        &self,
        taginfo: &TagInfo,
        variant: &Variant,
        field: &ast::Field<'a>,
    ) -> QuoteMaker {
        if field.attrs.skip_serializing() {
            return self.derive_unit_variant(taginfo, variant);
        }
        let ty = self.field_to_ts(field);
        let tag = ident_from_str(taginfo.tag);
        let content = if let Some(content) = taginfo.content {
            ident_from_str(&content)
        } else {
            ident_from_str(CONTENT)
        };
        let variant_name = self.variant_name(variant);
 
        quote! (
            { #tag: #variant_name, #content: #ty }
        )
        .into()
    }

    fn derive_struct_variant(
        &self,
        taginfo: &TagInfo,
        variant: &Variant,
        fields: &[ast::Field<'a>],
        container: &ast::Container,
    ) -> QuoteMaker {
        use std::collections::HashSet;
        let fields = filter_visible(fields);
        if fields.len() == 0 {
            return self.derive_unit_variant(taginfo, variant);
        }

        self.check_flatten(&fields, container);

        let contents = self.derive_fields(&fields);
        let variant_name = self.variant_name(variant);

        let tag = ident_from_str(taginfo.tag);
        if let Some(content) = taginfo.content {
            let content = ident_from_str(&content);
            quote! (
                { #tag: #variant_name, #content: { #(#contents),* } }
            )
            .into()
        } else {
            if let Some(ref cx) = self.ctxt {
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
            }
            quote! (
                { #tag: #variant_name, #(#contents),* }
            )
            .into()
        }
    }

    #[inline]
    fn variant_name(&self, variant: &Variant) -> String {
        variant.attrs.name().serialize_name()  // use serde name instead of variant.ident
    }

    fn derive_tuple_variant(
        &self,
        taginfo: &TagInfo,
        variant: &Variant,
        fields: &[ast::Field<'a>],
    ) -> QuoteMaker {
        let variant_name = self.variant_name(variant);
        let fields = filter_visible(fields);
        let contents = self.derive_field_types(&fields);

        let tag = ident_from_str(taginfo.tag);
        let content = if let Some(content) = taginfo.content {
            ident_from_str(&content)
        } else {
            ident_from_str(CONTENT)
        };

        quote! (
        { #tag: #variant_name, #content : [ #(#contents),* ] }
        )
        .into()
    }
}
