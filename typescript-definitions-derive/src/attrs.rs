// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::Ctxt;
use quote::quote;
// use std::str::FromStr;
// use syn::Type::Path;
use proc_macro2::TokenStream;
use syn::{Attribute, Ident, Lit, Meta, /* MetaList ,*/ MetaNameValue, NestedMeta};

#[derive(Debug)]
pub struct Attrs {
    pub comments: Vec<String>,
    pub verify: bool,
    pub turbo_fish: Option<TokenStream>,
}

#[inline]
fn path_to_str(path: &syn::Path) -> String {
    quote!(#path).to_string()
}

pub fn turbo_fish_check(v: &str) -> Result<TokenStream, String> {
    match v.parse::<proc_macro2::TokenStream>() {
        // just get LexError as error message... so make our own.
        Err(_) => Err(format!("Can't lex turbo_fish \"{}\"", v)),
        Ok(tokens) => match syn::parse2::<syn::DeriveInput>(quote!( struct S{ a:v#tokens} )) {
            Err(_) => Err(format!("Can't parse turbo_fish \"{}\"", v)),
            Ok(_) => Ok(tokens),
        },
    }
}
impl Attrs {
    pub fn new() -> Attrs {
        Attrs {
            comments: vec![],
            turbo_fish: None,
            verify: false,
        }
    }
    pub fn push_doc_comment(&mut self, attrs: &[Attribute]) {
        let doc_comments = attrs
            .iter()
            .filter_map(|attr| match path_to_str(&attr.path) == "doc" {
                true => attr.parse_meta().ok(),
                false => None,
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
        if self.comments.len() == 0 {
            String::default()
        } else {
            self.comments
                .iter()
                .map(|s| s.clone())
                .collect::<Vec<_>>()
                .join("\n")
                + "\n" // <-- need better way!
        }
    }
    fn err_msg(&self, msg: String) {
        panic!(msg);
    }
    pub fn find_typescript<'a>(
        attrs: &'a [Attribute],
        ctxt: &'a Ctxt,
    ) -> impl Iterator<Item = Meta> + 'a {
        use syn::Meta::*;
        use NestedMeta::*;

        attrs
            .iter()
            .filter_map(move |attr| match path_to_str(&attr.path).as_ref() {
                "typescript" => match attr.parse_meta() {
                    Ok(v) => Some(v),
                    Err(msg) => {
                        ctxt.error(format!("invalid typescript syntax: {}", msg));
                        None
                    }
                },
                _ => None,
            })
            .filter_map(move |m| match m {
                List(l) => Some(l.nested),
                ref tokens => {
                    ctxt.error(format!(
                        "unsupported syntax: {}",
                        quote!(#tokens).to_string()
                    ));
                    None
                }
            })
            .flatten()
            .filter_map(move |m| match m {
                Meta(m) => Some(m),
                ref tokens => {
                    ctxt.error(format!(
                        "unsupported syntax: {}",
                        quote!(#tokens).to_string()
                    ));
                    None
                }
            })
    }
    pub fn push_attrs(&mut self, struct_ident: &Ident, attrs: &[Attribute], ctxt: &Ctxt) {
        use syn::Meta::*;
        use Lit::*;
        // use NestedMeta::*;

        for attr in Self::find_typescript(&attrs, &ctxt) {
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
                            self.err_msg(format!(
                                "{}: verify must be true or false not \"{}\"",
                                struct_ident,
                                quote!(#value)
                            ));
                            false
                        }
                    }
                }
                Word(ref w) if w == "verify" => self.verify = true,
                NameValue(MetaNameValue {
                    ref ident,
                    lit: Str(ref value),
                    ..
                }) if ident == "turbo_fish" => {
                    let v = value.value();
                    match turbo_fish_check(&v) {
                        Err(msg) => self.err_msg(msg),
                        Ok(tokens) => self.turbo_fish = Some(tokens),
                    }
                }
                ref i @ NameValue(..) | ref i @ List(..) | ref i @ Word(..) => {
                    self.err_msg(format!("unsupported option: {}", quote!(#i)));
                }
            }
        }
    }
    /*
    fn push_generic_value(&mut self, ident: &Ident, lit: &Lit) {
        self.instance.push(Instance{ ident: quote!(#ident), value: quote!(#lit)} );

        use Lit::*;
        match lit {
        Str(
            ref token
        ) => {},
        ByteStr(
            ref token
        ) => {},
        Byte(
            ref token
        )=> {},
        Char(
            ref token
        )=> {},
        Int(
            ref token
        )=> {},
        Float(
            ref token
        )=> {},
        Bool(
            ref value
        )=> {},
        Verbatim(
            ref token
        )=> {},

    }

     pub fn to _generics(&self) -> TokenStream {
        if self.instance.is_empty() {
            return quote!();
        }
        let values = self.instance.iter().map(|i| &i.value);
        quote!(<#(#values),*>)
    } */
}
