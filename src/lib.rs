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

#[macro_use]
#[allow(unused_imports)]
extern crate lazy_static;
extern crate proc_macro2;
extern crate regex;
extern crate serde_derive_internals;
extern crate syn;

#[cfg(feature = "bytes")]
extern crate serde_bytes;

use proc_macro2::{Ident, Span};

use serde_derive_internals::{ast, Ctxt, Derive};
use std::str::FromStr;
use syn::DeriveInput;

mod derive_enum;
mod derive_struct;
mod patch;

// too many TokenStreams around! give it a different name
type QuoteT = proc_macro2::TokenStream;

struct Parsed {
    ident: syn::Ident,
    generics: Vec<Option<Ident>>,
    body: QuoteT,
}
impl Parsed {
    fn to_export_string(&self) -> String {
        let ts = self.body.to_string();
        let ts_ident = self.ts_ident().to_string();
        format!(
            "export type {} = {};",
            patch::patch(&ts_ident),
            patch::patch(&ts)
        )
    }

    fn ts_ident(&self) -> QuoteT {
        let ident = self.ident.clone();

        let args_wo_lt: Vec<_> = self
            .generics
            .iter()
            .filter_map(|g| g.clone())
            .map(|g| quote!(#g))
            .collect();
        if args_wo_lt.len() == 0 {
            quote!(#ident)
        } else {
            quote!(#ident<#(#args_wo_lt),*>)
        }
    }

    fn generic_args_wo_lifetimes(&self) -> impl Iterator<Item = QuoteT> + '_ {
        self.generics
            .iter()
            .filter_map(|g| g.clone())
            .map(|g| quote!(#g))
    }

    fn generic_args_with_lifetimes(&self) -> impl Iterator<Item = QuoteT> + '_ {
        self.generics.iter().map(|g| match g {
            Some(i) => quote!(#i),
            None => quote!('_),
        })
    }

    fn parse(input: proc_macro::TokenStream) -> Parsed {
        let input: DeriveInput = syn::parse(input).unwrap();

        let cx = Ctxt::new();
        let container = ast::Container::from_ast(&cx, &input, Derive::Serialize);

        let typescript: QuoteT = match container.data {
            ast::Data::Enum(ref variants) => derive_enum::derive_enum(&variants, &container),
            ast::Data::Struct(style, fields) => {
                derive_struct::derive_struct(style, &fields, &container.attrs)
            }
        };

        let generics = syn_generics(container.generics);

        // consumes context
        cx.check().unwrap();
        Parsed {
            ident: container.ident,
            generics: generics,
            body: typescript,
        }
    }
}

fn ident_from_str(s: &str) -> Ident {
    syn::Ident::new(s, Span::call_site())
}
/// derive proc_macro to expose typescript definitions to `wasm-bindgen`.
///
/// please see documentation at [crates.io](https://crates.io/crates/typescript-definitions)
///
#[proc_macro_derive(TypescriptDefinition)]
pub fn derive_typescript_definition(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if cfg!(any(debug_assertions, feature = "export-typescript")) {
        let parsed = Parsed::parse(input);
        let export_string = parsed.to_export_string();

        let export_ident = ident_from_str(&format!(
            "TS_EXPORT_{}",
            parsed.ident.to_string().to_uppercase()
        ));

        // eprintln!(
        //     "....[typescript] export type {}={};",
        //     parsed.ident, typescript_string
        // );
        let mut q = quote! {

            #[wasm_bindgen(typescript_custom_section)]
            pub const #export_ident : &'static str = #export_string;
        };

        if cfg!(any(test, feature = "test")) {
            let typescript_ident =
                ident_from_str(&format!("{}___typescript_definition", &parsed.ident));
            let ts = proc_macro2::TokenStream::from_str(&export_string)
                .unwrap()
                .to_string()
                .replace("\n", " ");

            q.extend(quote!(
                fn #typescript_ident ( ) -> &'static str {
                   #ts
                }

            ));
        }

        q.into()
    } else {
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
        let parsed = Parsed::parse(input);
        let export_string = parsed.to_export_string();

        let ident = parsed.ident.clone();
        let ret = if parsed.generics.len() == 0 {
            quote! {

                impl TypeScriptifyTrait for #ident {
                    fn type_script_ify() ->  &'static str {
                        #export_string
                    }
                }
            }
        } else {
            let generics = parsed.generic_args_with_lifetimes();
            let implg = parsed.generic_args_wo_lifetimes();
            quote! {

                impl<#(#implg),*> TypeScriptifyTrait for #ident<#(#generics),*> {
                    fn type_script_ify() ->  &'static str {
                        #export_string
                    }
                }
            }
        };
        // eprintln!("{}", ret.to_string());

        ret.into()
    } else {
        proc_macro::TokenStream::new()
    }
}

fn syn_generics(g: &syn::Generics) -> Vec<Option<Ident>> {
    // get all the generics
    // we ignore type parameters because we can't
    // reasonably serialize generic structs! But e.g.
    // std::borrow::Cow; requires a lifetime parameter ... see tests/typescript.rs
    use syn::{ConstParam, GenericParam, LifetimeDef, TypeParam};
    g.params
        .iter()
        .map(|p| match p {
            GenericParam::Lifetime(LifetimeDef { /* lifetime,*/ .. }) => None,
            GenericParam::Type(TypeParam { ident, ..}) => Some(ident.clone()),
            GenericParam::Const(ConstParam { ident, ..}) => Some(ident.clone()),

        })
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
            // see patch.rs...
            quote!(  { Ok : #k } __ZZ__patch_me__ZZ__ { Err : #v }  )
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
