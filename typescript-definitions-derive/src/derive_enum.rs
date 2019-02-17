// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use quote::quote;
use serde_derive_internals::{ast, ast::Variant, attr::EnumTag};
use super::patch::{TRIPPLE_EQ, NL_PATCH};
use super::{filter_visible, ident_from_str, ParseContext, QuoteMaker, Attrs, verify::Verify};
use proc_macro2::Literal;
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
            let obj = &self.verify;

            let verify = quote!(
                {
                    if (#obj == undefined) return false; 
                    if (![#(#v),*].includes(#obj)) return false; 
                    return true;
                }
            );

            return QuoteMaker {
                body: quote! ( { #(#k = #v),* } ),
                verify: Some(verify),
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
        }).collect::<Vec<_>>();
        // OK generate A | B | C etc
        let body = content.iter().map(|q| q.body.clone());
        let verify = content.iter().map(|q| q.verify.clone().unwrap());
        let obj = & self.verify;
        let p = content.iter().map(|_| Literal::string(NL_PATCH));
        let verify = quote!(
            {
                if (#obj == undefined) return false;

                #( #p if ( ( () => #verify )() ) return true; )*
                return false;
            }
        ); 
        QuoteMaker {
            body: quote! ( #(|#body)* ),
            verify: Some(verify),
            is_enum: false,
        }
    }
    fn derive_unit_variant(&self, taginfo: &TagInfo, variant: &Variant) -> QuoteMaker {
        let variant_name = variant.attrs.name().serialize_name(); // use serde name instead of variant.ident
        let eq = ident_from_str(TRIPPLE_EQ);       
        
        if taginfo.tag.is_none() {
            let obj = &self.verify;
            let verify = quote!(
                {
                    return #obj #eq #variant_name;
                }
            );
            return QuoteMaker {
                body: quote!(#variant_name),
                verify: Some(verify),
                is_enum: false,
            };
        }
        let tag = ident_from_str(taginfo.tag.unwrap());
        let obj = &self.verify;
        let verify = quote!(
            {
                return #obj.#tag #eq #variant_name;
            }
        );
        QuoteMaker {
            body: quote! (
                { #tag: #variant_name }
            ),
            verify: Some(verify),
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
        let attrs = Attrs::from_field(field, self.ctxt);
        let verify = Verify {
            attrs,
            ctxt: self,
            field: field,
        };
        let obj = &self.verify;

        if taginfo.tag.is_none() {
            
            if taginfo.untagged {
                let verify = verify.verify_type(&obj, &field.ty);
                return QuoteMaker {
                    body: quote! ( #ty ),
                    verify: Some(quote!( { #verify; return true; } )),
                    is_enum: false,
                };
            };
            let tag = ident_from_str(&variant_name);
            let v = quote!(v);
            let verify = verify.verify_type(&v, &field.ty);
            let verify = quote!(
                {
                    let v = #obj.#tag;
                    if (v == undefined) return false;
                    #verify;
                    return true;
                }
            );
            return QuoteMaker {
                body: quote! (
                    { #tag : #ty }

                ),
                verify: Some(verify),
                is_enum: false,
            };
        };
        let tag = ident_from_str(taginfo.tag.unwrap());

        let content = if let Some(content) = taginfo.content {
            ident_from_str(&content)
        } else {
            ident_from_str(CONTENT) // should not get here...
        };

        let eq = ident_from_str(TRIPPLE_EQ);
        let verify = verify.verify_type(&quote!(v), &field.ty);
        let verify = quote!(
            {
                if (!(#obj.#tag #eq #variant_name)) return false;
                let v = #obj.#content;
                if (v == undefined) return false;
                #verify;
                return true;
            }
        );
        QuoteMaker {
            body: quote! (
                { #tag: #variant_name; #content: #ty }
            ),
            verify: Some(verify),
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
                let v = self.verify_fields(&self.verify, &fields);

                let verify = quote!(
                    {
                        #(#v;)*
                        return true;
                    }
                );
                return QuoteMaker {
                    body: quote! (
                        { #(#contents);* }
                    ),
                    verify: Some(verify),
                    is_enum: false,
                };
            };
            let v = &quote!(v);
            let tag = ident_from_str(&variant_name);
            let obj = &self.verify;
            let v = self.verify_fields(&v, &fields);
            let verify = quote!(
                {
                    let v = #obj.#tag;
                    if (v == undefined) return false;
                    #(#v;)*
                    return true;
                }
                );
            return QuoteMaker {
                body: quote! (
                    { #tag : { #(#contents);* }  }
                ),
                verify: Some(verify),
                is_enum: false,
            };
        }
        let tag_str = taginfo.tag.unwrap();
        let tag = ident_from_str(tag_str);
        let obj = &self.verify;
        let v = &quote!(v);
        let v = self.verify_fields(&v, &fields);
        let eq = ident_from_str(TRIPPLE_EQ);

        if let Some(content) = taginfo.content {
            let content = ident_from_str(&content);
            let verify = quote!(
            {
                if (!(#obj.#tag #eq #variant_name)) return false;
                let v = #obj.#content;
                if (v == undefined) return false;
                #(#v;)*
                return true;
            }
            );
            QuoteMaker {
                body: quote! (
                    { #tag: #variant_name; #content: { #(#contents);* } }

                ),
                verify: Some(verify),
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
            };
            let verify = quote!(
            {
                if (!(#obj.#tag #eq #variant_name)) return false;
                let v = #obj;
                #(#v;)*
                return true;
            }
            );
            QuoteMaker {
                body: quote! (
                    { #tag: #variant_name; #(#contents);* }
                ),
                verify: Some(verify),
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
        let obj = &self.verify;
        let v = &quote!(v);
        let v = self.verify_field_tuple(&v, &fields);
        
        if taginfo.tag.is_none() {
            if taginfo.untagged {
                let verify = quote!({
                    const v = #obj;
                    #(#v;)*
                    return true;
                });
                return QuoteMaker {
                    body: quote! (
                     [ #(#contents),* ]
                    ),
                    verify: Some(verify),
                    is_enum: false,
                };
            }
            let tag = ident_from_str(&variant_name);
            let verify = quote!({
                const v = #obj.#tag;
                if (v == undefined) return true;
                #(#v;)*
                return true;
            });
            return QuoteMaker {
                body: quote! (
                 { #tag : [ #(#contents),* ] }

                ),
                verify: Some(verify),
                is_enum: false,
            };
        };

        let tag = ident_from_str(taginfo.tag.unwrap());
        let content = if let Some(content) = taginfo.content {
            ident_from_str(&content)
        } else {
            ident_from_str(CONTENT)
        };
        let eq = ident_from_str(TRIPPLE_EQ);
        let verify = quote!({
            if (!(#obj.tag #eq #variant_name)) return false;
            const v = #obj.#content;
            if (v == undefined) return true;
            #(#v;)*
            return true;
        });
        QuoteMaker {
            body: quote! (
            { #tag: #variant_name; #content : [ #(#contents),* ] }
            ),
            verify: Some(verify),
            is_enum: false,
        }
    }
}
