// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Exports serde-serializable structs and enums to Typescript definitions.
//! 
//! please see documentation at [crates.io](https://crates.io/crates/typescript-definitions)


extern crate proc_macro;

#[macro_use]
extern crate quote;

// extern crate serde;
extern crate proc_macro2;
extern crate regex;
extern crate serde_derive_internals;
extern crate syn;
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "bytes")]
extern crate serde_bytes;

use proc_macro2::Span;
// use quote::TokenStreamExt;

use serde_derive_internals::{ast, Ctxt, Derive};
use syn::DeriveInput;
// use proc_macro::TokenStream;
// use syn::Meta::{List, NameValue, Word};
// use syn::NestedMeta::{Literal, Meta};


mod derive_enum;
mod derive_struct;

type QuoteT = proc_macro2::TokenStream;

mod patch {
    use regex::{Captures, Regex};
    use std::borrow::Cow;
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(?P<nl>\n+)|(?P<brack>\s*\[\s+\])|(?P<brace>\{\s+\})|(?P<colon>\s[:]\s)").unwrap();
    }
    // TODO: where does the newline come from? why the double spaces?

    trait Has {
        fn has(&self, s: &'static str) -> bool;
    }

    impl Has for Captures<'_> {
        #[inline]
        fn has(&self, s: &'static str) -> bool {
            self.name(s).is_some()
        }
    }
   

    pub fn debug_patch<'t>(s: &'t str) -> Cow<'t, str> {
        RE.replace_all(s, |c: &Captures| {
            // c.get(0).map(|s| s.)
            if c.has("brace") {
                "{ }"
            } else if c.has("brack") {
                " [ ]"
            } else if c.has("colon") {
                " : "
            } else {
                assert!(c.has("nl"));
                " "
            }
        })
    }

    pub fn patch<'t>(s: &'t str) -> Cow<'t, str> {
        RE.replace_all(s, |c: &Captures| {
            if c.has("brace") {
                "{}"
            } else if c.has("brack") {
                "[]"
            } else if c.has("colon") {
                ": "
            } else {
                assert!(c.has("nl"));
                "\n"
            }
        })
    }
}
/*
fn get_ts_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "ts" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                // TODO: produce an error
                None
            }
        }
    } else {
        None
    }
}
*/

struct Parsed {
    ident: syn::Ident,
    lifetimes: Vec<QuoteT>,
    body: QuoteT,
}

fn parse(input: proc_macro::TokenStream) -> Parsed {
    // eprintln!(".........[input] {}", input);
    let input: DeriveInput = syn::parse(input).unwrap();
    /*
    let mut astagged = false;
    for meta in input.attrs.iter().filter_map(get_ts_meta_items) {
        for meta_item in meta {
            match meta_item {
                Meta(Word(ref word)) if word == "astagged" => {
                            astagged = true;
                        }
                _ => {}
            }
        }

    }*/

    let cx = Ctxt::new();
    let container = ast::Container::from_ast(&cx, &input, Derive::Serialize);

    let typescript: QuoteT = match container.data {
        ast::Data::Enum(variants) => derive_enum::derive_enum(&variants, &container.attrs),
        ast::Data::Struct(style, fields) => {
            derive_struct::derive_struct(style, &fields, &container.attrs)
        }
    };

    let lifetimes = generic_lifetimes(container.generics);

    // consumes context
    cx.check().unwrap();
    Parsed {
        ident: container.ident,
        lifetimes: lifetimes,
        body: typescript,
    }
}

fn ident_from_str(s: &str) -> proc_macro2::Ident {
    syn::Ident::new(s, Span::call_site())
}
/// derive proc_macro to expose typescript definitions to `wasm-bindgen`.
///
/// please see documentation at [crates.io](https://crates.io/crates/typescript-definitions)
///

#[proc_macro_derive(TypescriptDefinition)]
pub fn derive_typescript_definition(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    if cfg!(any(debug_assertions, feature = "export-typescript")) {
        let parsed = parse(input);

        let typescript_string = parsed.body.to_string();
        let export_string = format!(
            "export type {} = {};",
            parsed.ident,
            patch::patch(&typescript_string)
        );

        let export_ident = ident_from_str(&format!(
            "TS_EXPORT_{}",
            parsed.ident.to_string().to_uppercase()
        ));

        // eprintln!(
        //     "....[typescript] export type {}={};",
        //     parsed.ident, typescript_string
        // );
        let mut q  = quote! {

            #[wasm_bindgen(typescript_custom_section)]
            pub const #export_ident : &'static str = #export_string;
        };
        
        if cfg!(any(test,feature="test")) {
            let ts = patch::debug_patch(&typescript_string); // why the newlines?
            let typescript_ident = ident_from_str(&format!("{}___typescript_definition", parsed.ident));
   
            q.extend(
                quote!(

                fn #typescript_ident ( ) -> &'static str {
                    #ts
                }
            
            ));
        }
        
        q.into()
    
    } else {
        //proc_macro2::TokenStream::new().into()
        proc_macro::TokenStream::new()
    }

}

/// derive proc_macro to expose typescript definitions to as a static function.
///
/// please see documentation at [crates.io](https://crates.io/crates/typescript-definitions)
/// 
#[proc_macro_derive(TypeScriptify)]
pub fn derive_type_script_ify(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    if cfg!(any(debug_assertions, feature = "export-typescript")) {
        let parsed = parse(input);
        let ts = parsed.body.to_string();
        let export_string = format!("export type {} = {} ;", parsed.ident, patch::patch(&ts));
        let ident = parsed.ident;

        let ret = if parsed.lifetimes.len() == 0 {
            quote! {

                impl TypeScriptifyTrait for #ident {
                    fn type_script_ify() ->  &'static str {
                        #export_string
                    }
                }
            }
        } else {
            // can't use 'a need '_
            let lt = parsed.lifetimes.iter().map(|_q| quote!('_));
            quote! {

                impl TypeScriptifyTrait for #ident<#(#lt),*> {
                    fn type_script_ify() ->  &'static str {
                        #export_string
                    }
                }
            }
        };

        ret.into()
    } else {
        proc_macro::TokenStream::new()
    }
}


fn generic_lifetimes(g: &syn::Generics) -> Vec<QuoteT> {
    // get all the generic lifetimes
    // we ignore type parameters because we can't
    // reasonably serialize generic structs! But e.g.
    // std::borrow::Cow; requires a lifetime parameter ... see tests/typescript.rs
    use syn::{GenericParam, LifetimeDef};
    g.params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => Some(lifetime),
            _ => None,
        })
        .map(|lt| quote!(#lt))
        .collect()
}

fn return_type(rt: &syn::ReturnType) -> Option<QuoteT> {
    match rt {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, tp) => Some(type_to_ts(tp)),
    }
}

struct TSType {
    ident: syn::Ident,
    args: Vec<QuoteT>,
}
fn last_path_element(path: &syn::Path) -> Option<TSType> {
    match path.segments.last().map(|p| p.into_value()) {
        Some(t) => {
            let ident = t.ident.clone();
            let args = match &t.arguments {
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) => args,
                // turn func(A,B) ->C into  func<C>?
                syn::PathArguments::Parenthesized(syn::ParenthesizedGenericArguments {
                    output,
                    ..
                }) => {
                    let args = if let Some(rt) = return_type(output) {
                        vec![rt]
                    } else {
                        vec![]
                    };
                    return Some(TSType {
                        ident: ident,
                        args: args,
                    });
                }
                _ => {
                    return Some(TSType {
                        ident: ident,
                        args: vec![],
                    })
                }
            };
            // ignore lifetimes
            let args = args
                .iter()
                .filter_map(|p| match p {
                    syn::GenericArgument::Type(t) => Some(t),
                    _ => None,
                })
                .map(|p| type_to_ts(p))
                .collect::<Vec<_>>();

            Some(TSType {
                ident: ident,
                args: args,
            })
        }
        None => None,
    }
}
fn generic_to_ts(ts: TSType) -> QuoteT {
    match ts.ident.to_string().as_ref() {
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128"
        | "isize" | "f64" | "f32" => quote! { number },
        "String" | "str" => quote! { string },
        "bool" => quote! { boolean },
        "Cow" | "Rc" | "Arc" if ts.args.len() == 1 => ts.args[0].clone(),

        // std::collections
        "Vec" | "VecDeque" | "LinkedList" if ts.args.len() == 1 => {
            let t = &ts.args[0];
            quote! { #t[] }
        }
        "HashMap" | "BTreeMap" if ts.args.len() == 2 => {
            let k = &ts.args[0];
            let v = &ts.args[1];
            quote!(Map<#k,#v>)
        }
        "HashSet" | "BTreeSet" if ts.args.len() == 1 => {
            let k = &ts.args[0];
            quote!(Set<#k>)
        }
        "Option" if ts.args.len() == 1 => {
            let k = &ts.args[0];
            quote!(  #k | null  )
        }
        "Result" if ts.args.len() == 2 => {
            let k = &ts.args[0];
            let v = &ts.args[1];
            // TODO what if k or v is A | B | C ?
            // maybe A | B | C | #v is actually better than (A|B|C) | #v
            quote!(  #k | #v  )
        }
        _ => {
            let ident = ts.ident;
            if ts.args.len() > 0 {
                let args = ts.args;
                quote! { #ident<#(#args),*> }
            } else {
                quote! {#ident}
            }
        }
    }
}

fn type_to_ts(ty: &syn::Type) -> QuoteT {
    fn type_to_array(elem: &syn::Type) -> QuoteT {
        let tp = type_to_ts(elem);
        quote! { #tp[] }
    };

    use syn::Type::*;
    use syn::{
        TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeParamBound, TypeParen, TypePath,
        TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
    };
    match ty {
        Slice(TypeSlice { elem, .. }) => type_to_array(elem),
        Array(TypeArray { elem, .. }) => type_to_array(elem),
        Ptr(TypePtr { elem, .. }) => type_to_array(elem),
        Reference(TypeReference { elem, .. }) => type_to_ts(elem),
        // fn(A,B,C) -> D to D?
        BareFn(TypeBareFn { output, .. }) => {
            if let Some(rt) = return_type(&output) {
                rt
            } else {
                quote!(undefined)
            }
        }
        Never(..) => quote! { never },
        Tuple(TypeTuple { elems, .. }) => {
            let elems = elems.iter().map(|t| type_to_ts(t));
            quote!([ #(#elems),* ])
        }

        Path(TypePath { path, .. }) => match last_path_element(&path) {
            Some(ts) => generic_to_ts(ts),
            _ => quote! { any },
        },
        TraitObject(TypeTraitObject { bounds, .. }) | ImplTrait(TypeImplTrait { bounds, .. }) => {
            let elems = bounds
                .iter()
                .filter_map(|t| match t {
                    TypeParamBound::Trait(t) => last_path_element(&t.path),
                    _ => None, // skip lifetime etc.
                })
                .map(|t| {
                    let ident = t.ident;
                    quote!(#ident)
                });

            // TODO check for zero length?
            quote!(#(#elems)|*)
        }
        Paren(TypeParen { elem, .. }) => {
            let tp = type_to_ts(elem);
            quote! { ( #tp ) }
        }
        Group(TypeGroup { elem, .. }) => type_to_ts(elem),
        Infer(..) | Macro(..) | Verbatim(..) => quote! { any },
    }
}

fn derive_field<'a>(field: &ast::Field<'a>) -> QuoteT {
    let field_name = field.attrs.name().serialize_name();
    let field_name = ident_from_str(&field_name);

    let ty = type_to_ts(&field.ty);
    quote! {
        #field_name: #ty
    }
}
