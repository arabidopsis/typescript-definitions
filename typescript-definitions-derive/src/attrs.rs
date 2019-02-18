// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{ast, ident_from_str, Ctxt};
use quote::quote;
use std::collections::HashMap;
// use std::str::FromStr;
// use syn::Type::Path;
use proc_macro2::TokenStream;
use syn::{Attribute, Ident, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

#[derive(Debug)]
pub struct Attrs {
    pub comments: Vec<String>,
    pub verify: bool,
    pub turbofish: Option<TokenStream>,
    pub only_first: bool,
    pub isa: HashMap<String, TokenStream>,
}

#[inline]
fn path_to_str(path: &syn::Path) -> String {
    quote!(#path).to_string()
}

pub fn turbofish_check(v: &str) -> Result<TokenStream, String> {
    match v.parse::<proc_macro2::TokenStream>() {
        // just get LexError as error message... so make our own.
        Err(_) => Err(format!("Can't lex turbofish \"{}\"", v)),
        Ok(tokens) => match syn::parse2::<syn::DeriveInput>(quote!( struct S{ a:v#tokens} )) {
            Err(_) => Err(format!("Can't parse turbofish \"{}\"", v)),
            Ok(_) => Ok(tokens),
        },
    }
}
impl Attrs {
    pub fn new() -> Attrs {
        Attrs {
            comments: vec![],
            turbofish: None,
            verify: false,
            only_first: false,
            isa: HashMap::new(),
        }
    }
    pub fn push_doc_comment(&mut self, attrs: &[Attribute]) {
        let doc_comments = attrs
            .iter()
            .filter_map(|attr| {
                if path_to_str(&attr.path) == "doc" {
                    attr.parse_meta().ok()
                } else {
                    None
                }
            })
            .filter_map(|attr| {
                use Lit::*;
                use Meta::*;
                if let NameValue(MetaNameValue {
                    ident, lit: Str(s), ..
                }) = attr
                {
                    if ident != "doc" {
                        return None;
                    }
                    let value = s.value();
                    let text = value
                        .trim_start_matches("//!")
                        .trim_start_matches("///")
                        .trim_start_matches("/*!")
                        .trim_start_matches("/**")
                        .trim_end_matches("*/")
                        .trim();
                    if text.is_empty() {
                        None
                    } else {
                        Some(text.to_string())
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if doc_comments.is_empty() {
            return;
        }

        let merged_lines = doc_comments
            .iter()
            .map(|s| format!("// {}", s))
            .collect::<Vec<_>>()
            .join("\n");

        self.comments.push(merged_lines);
    }

    pub fn to_comment_str(&self) -> String {
        if self.comments.is_empty() {
            String::default()
        } else {
            self.comments.join("\n") + "\n" // <-- need better way!
        }
    }
    fn err_msg<'a>(&self, msg: String, ctxt: Option<&'a Ctxt>) {
        if let Some(ctxt) = ctxt {
            ctxt.error(msg);
        } else {
            panic!(msg)
        };
    }
    pub fn find_typescript<'a>(
        attrs: &'a [Attribute],
        ctxt: Option<&'a Ctxt>,
    ) -> impl Iterator<Item = Meta> + 'a {
        use syn::Meta::*;
        use NestedMeta::*;

        attrs
            .iter()
            .filter_map(move |attr| match path_to_str(&attr.path).as_ref() {
                "typescript" => match attr.parse_meta() {
                    Ok(v) => Some(v),
                    Err(msg) => {
                        if let Some(ctxt) = ctxt {
                            ctxt.error(format!("invalid typescript syntax: {}", msg));
                        } else {
                            panic!("invalid typescript syntax: {}", msg)
                        };
                        None
                    }
                },
                _ => None,
            })
            .filter_map(move |m| match m {
                List(l) => Some(l.nested),
                ref tokens => {
                    if let Some(ctxt) = ctxt {
                        ctxt.error(format!(
                            "unsupported syntax: {}",
                            quote!(#tokens).to_string()
                        ));
                    } else {
                        panic!("invalid typescript syntax: {}", quote!(#tokens).to_string())
                    };
                    None
                }
            })
            .flatten()
            .filter_map(move |m| match m {
                Meta(m) => Some(m),
                ref tokens => {
                    if let Some(ctxt) = ctxt {
                        ctxt.error(format!(
                            "unsupported syntax: {}",
                            quote!(#tokens).to_string()
                        ));
                    } else {
                        panic!("invalid typescript syntax: {}", quote!(#tokens).to_string())
                    };
                    None
                }
            })
    }
    pub fn push_attrs(&mut self, struct_ident: &Ident, attrs: &[Attribute], ctxt: Option<&Ctxt>) {
        use syn::Meta::*;
        use Lit::*;
        use NestedMeta::*;

        for attr in Self::find_typescript(&attrs, ctxt) {
            match attr {
                NameValue(MetaNameValue {
                    ref ident,
                    lit: Bool(ref value),
                    ..
                }) if ident == "verify" => {
                    self.verify = value.value;
                }
                NameValue(MetaNameValue {
                    ref ident,
                    lit: Str(ref value),
                    ..
                }) if ident == "verify" => {
                    self.verify = match value.value().parse() {
                        Ok(v) => v,
                        Err(..) => {
                            self.err_msg(
                                format!(
                                    "{}: verify must be true or false not \"{}\"",
                                    struct_ident,
                                    quote!(#value)
                                ),
                                ctxt,
                            );
                            false
                        }
                    }
                }
                List(MetaList {
                    ref ident,
                    ref nested,
                    ..
                }) if ident == "isa" => {
                    for method in nested {
                        match *method {
                            Meta(NameValue(MetaNameValue {
                                ref ident,
                                lit: Str(ref v),
                                ..
                            })) => match v.value().parse::<TokenStream>() {
                                Ok(t) => {
                                    self.isa.insert(ident.to_string(), quote!(#t));
                                }
                                Err(_) => self.err_msg(format!("Can't parse {}", quote!(#v)), ctxt),
                            },
                            ref mi @ _ => panic!("unsupported raw entry: {}", quote!(#mi)),
                        }
                    }
                }
                Word(ref w) if w == "verify" => self.verify = true,
                NameValue(MetaNameValue {
                    ref ident,
                    lit: Str(ref value),
                    ..
                }) if ident == "turbofish" => {
                    let v = value.value();
                    match turbofish_check(&v) {
                        Err(msg) => self.err_msg(msg, ctxt),
                        Ok(tokens) => self.turbofish = Some(tokens),
                    }
                }
                ref i @ NameValue(..) | ref i @ List(..) | ref i @ Word(..) => {
                    self.err_msg(format!("unsupported option: {}", quote!(#i)), ctxt);
                }
            }
        }
    }
    pub fn push_field_attrs(
        &mut self,
        struct_ident: &Ident,
        attrs: &[Attribute],
        ctxt: Option<&Ctxt>,
    ) {
        use syn::Meta::*;
        use Lit::*;
        // use NestedMeta::*;

        for attr in Self::find_typescript(&attrs, ctxt) {
            match attr {
                NameValue(MetaNameValue {
                    ref ident,
                    lit: Str(ref value),
                    ..
                }) if ident == "check" => {
                    self.only_first = match value.value().as_ref() {
                        "first" => true,
                        "all" => false,
                        _ => {
                            self.err_msg(
                                format!(
                                    r#"{}: check value must be "first" or "all" not "{}""#,
                                    struct_ident,
                                    quote!(#value)
                                ),
                                ctxt,
                            );
                            false
                        }
                    }
                }
                ref i @ NameValue(..) | ref i @ List(..) | ref i @ Word(..) => {
                    self.err_msg(format!("unsupported option: {}", quote!(#i)), ctxt);
                }
            }
        }
    }
    pub fn from_field(field: &ast::Field, ctxt: Option<&Ctxt>) -> Attrs {
        let mut res = Self::new();
        if let Some(ref ident) = field.original.ident {
            res.push_field_attrs(ident, &field.original.attrs, ctxt);
        } else {
            let id = ident_from_str("unnamed");
            res.push_field_attrs(&id, &field.original.attrs, ctxt);
        }
        res
    }
}
