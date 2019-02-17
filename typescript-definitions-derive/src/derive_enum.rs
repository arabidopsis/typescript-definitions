// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use quote::quote;
use serde_derive_internals::{ast, ast::Variant, attr::EnumTag};

use super::{filter_visible, ident_from_str, ParseContext, QuoteMaker};

const CONTENT: &str = "fields"; // default content tag
                                // const TAG: &'static str = "kind"; // default tag tag

struct TagInfo<'a> {
    tag: Option<&'a str>,
    content: Option<&'a str>,
    untagged: bool,
}
impl<'a> ParseContext<'_> {
    pub(crate) fn derive_enum(
        &self,
        variants: &[ast::Variant<'a>],
        ast_container: &ast::Container,
    ) -> QuoteMaker {
        let taginfo = match ast_container.attrs.tag() {
            EnumTag::Internal { tag, .. } => TagInfo {
                tag: Some(tag),
                content: None,
                untagged: false,
            },
            EnumTag::Adjacent { tag, content, .. } => TagInfo {
                tag: Some(tag),
                content: Some(&content),
                untagged: false,
            },
            EnumTag::External => TagInfo {
                tag: None,
                content: None,
                untagged: false,
            },
            EnumTag::None => TagInfo {
                tag: None,
                content: None,
                untagged: true,
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
            let v = &skip_variants
                .iter()
                .map(|v| v.attrs.name().serialize_name()) // use serde name instead of v.ident
                .collect::<Vec<_>>();
            let k = v.iter().map(|v| ident_from_str(&v)).collect::<Vec<_>>();

            return QuoteMaker {
                body: quote! ( { #(#k = #v),* } ),
                verify: None,
                is_enum: true,
            };
        }

        let content = skip_variants.iter().map(|variant| match variant.style {
            ast::Style::Struct => {
                self.derive_struct_variant(&taginfo, variant, &variant.fields, ast_container)
            }
            ast::Style::Newtype => {
                self.derive_newtype_variant(&taginfo, variant, &variant.fields[0])
            }
            ast::Style::Tuple => self.derive_tuple_variant(&taginfo, variant, &variant.fields),
            ast::Style::Unit => self.derive_unit_variant(&taginfo, variant),
        });
        // OK generate A | B | C etc
        let body = content.map(|q| q.body);
        QuoteMaker {
            body: quote! ( #(|#body)* ),
            verify: None,
            is_enum: false,
        }
    }
    fn derive_unit_variant(&self, taginfo: &TagInfo, variant: &Variant) -> QuoteMaker {
        let variant_name = variant.attrs.name().serialize_name(); // use serde name instead of variant.ident
        if taginfo.tag.is_none() {
            return QuoteMaker {
                body: quote!(#variant_name),
                verify: None,
                is_enum: false,
            };
        }
        let tag = ident_from_str(taginfo.tag.unwrap());
        QuoteMaker {
            body: quote! (
                { #tag: #variant_name }
            ),
            verify: None,
            is_enum: false,
        }
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
        let variant_name = self.variant_name(variant);
        if taginfo.tag.is_none() {
            if taginfo.untagged {
                return QuoteMaker {
                    body: quote! ( #ty ),
                    verify: None,
                    is_enum: false,
                };
            };
            let tag = ident_from_str(&variant_name);
            return QuoteMaker {
                body: quote! (
                    { #tag : #ty }

                ),
                verify: None,
                is_enum: false,
            };
        };
        let tag = ident_from_str(taginfo.tag.unwrap());

        let content = if let Some(content) = taginfo.content {
            ident_from_str(&content)
        } else {
            ident_from_str(CONTENT) // should not get here...
        };

        QuoteMaker {
            body: quote! (
                { #tag: #variant_name; #content: #ty }
            ),
            verify: None,
            is_enum: false,
        }
    }

    fn derive_struct_variant(
        &self,
        taginfo: &TagInfo,
        variant: &Variant,
        fields: &[ast::Field<'a>],
        ast_container: &ast::Container,
    ) -> QuoteMaker {
        use std::collections::HashSet;
        let fields = filter_visible(fields);
        if fields.is_empty() {
            return self.derive_unit_variant(taginfo, variant);
        }

        self.check_flatten(&fields, ast_container);

        let contents = self.derive_fields(&fields);
        let variant_name = self.variant_name(variant);
        if taginfo.tag.is_none() {
            if taginfo.untagged {
                return QuoteMaker {
                    body: quote! (
                        { #(#contents);* }
                    ),
                    verify: None,
                    is_enum: false,
                };
            };
            let tag = ident_from_str(&variant_name);
            return QuoteMaker {
                body: quote! (
                    { #tag : { #(#contents);* }  }
                ),
                verify: None,
                is_enum: false,
            };
        }
        let tag_str = taginfo.tag.unwrap();
        let tag = ident_from_str(tag_str);
        if let Some(content) = taginfo.content {
            let content = ident_from_str(&content);
            QuoteMaker {
                body: quote! (
                    { #tag: #variant_name; #content: { #(#contents);* } }

                ),
                verify: None,
                is_enum: false,
            }
        } else {
            if let Some(ref cx) = self.ctxt {
                let fnames = fields
                    .iter()
                    .map(|field| field.attrs.name().serialize_name())
                    .collect::<HashSet<_>>();
                if fnames.contains(tag_str) {
                    cx.error(format!(
                        "clash with field in \"{}::{}\". \
                         Maybe use a #[serde(content=\"...\")] attribute.",
                        ast_container.ident, variant_name
                    ));
                }
            }
            QuoteMaker {
                body: quote! (
                    { #tag: #variant_name; #(#contents);* }
                ),
                verify: None,
                is_enum: false,
            }
        }
    }

    #[inline]
    fn variant_name(&self, variant: &Variant) -> String {
        variant.attrs.name().serialize_name() // use serde name instead of variant.ident
    }

    fn derive_tuple_variant(
        &self,
        taginfo: &TagInfo,
        variant: &Variant,
        fields: &[ast::Field<'a>],
    ) -> QuoteMaker {
        let variant_name = self.variant_name(variant);
        let fields = filter_visible(fields);
        let contents = self.derive_field_tuple(&fields);
        if taginfo.tag.is_none() {
            if taginfo.untagged {
                return QuoteMaker {
                    body: quote! (
                     [ #(#contents),* ]
                    ),
                    verify: None,
                    is_enum: false,
                };
            }
            let tag = ident_from_str(&variant_name);
            return QuoteMaker {
                body: quote! (
                 { #tag : [ #(#contents),* ] }

                ),
                verify: None,
                is_enum: false,
            };
        };

        let tag = ident_from_str(taginfo.tag.unwrap());
        let content = if let Some(content) = taginfo.content {
            ident_from_str(&content)
        } else {
            ident_from_str(CONTENT)
        };

        QuoteMaker {
            body: quote! (
            { #tag: #variant_name; #content : [ #(#contents),* ] }
            ),
            verify: None,
            is_enum: false,
        }
    }
}
