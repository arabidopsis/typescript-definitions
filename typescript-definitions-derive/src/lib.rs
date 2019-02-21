// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Exports serde-serializable structs and enums to Typescript definitions.
//!
//! Please see documentation at [crates.io](https://crates.io/crates/typescript-definitions)

extern crate proc_macro;

use proc_macro2::Ident;
use quote::quote;
use serde_derive_internals::{ast, Ctxt, Derive};
// use std::str::FromStr;
use std::cell::Cell;
use syn::DeriveInput;

mod attrs;
mod derive_enum;
mod derive_struct;
mod patch;
// mod quotet;
mod tests;
mod utils;
mod guards;
mod tots;

use attrs::Attrs;
use utils::*;

use patch::patch;


// too many TokenStreams around! give it a different name
type QuoteT = proc_macro2::TokenStream;

//type QuoteMaker = quotet::QuoteT<'static>;

type Bounds = Vec<TSType>;

struct QuoteMaker {
    pub body: QuoteT,
    pub verify: Option<QuoteT>,
    pub is_enum: bool,
}

/// derive proc_macro to expose Typescript definitions to `wasm-bindgen`.
///
/// Please see documentation at [crates.io](https://crates.io/crates/typescript-definitions).
///
#[proc_macro_derive(TypescriptDefinition, attributes(typescript))]
pub fn derive_typescript_definition(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if cfg!(any(debug_assertions, feature = "export-typescript")) {
        let input = QuoteT::from(input);
        do_derive_typescript_definition(input).into()
    } else {
        proc_macro::TokenStream::new()
    }
}
/// derive proc_macro to expose Typescript definitions as a static function.
///
/// Please see documentation at [crates.io](https://crates.io/crates/typescript-definitions).
///
#[proc_macro_derive(TypeScriptify, attributes(typescript))]
pub fn derive_type_script_ify(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if cfg!(any(debug_assertions, feature = "export-typescript")) {
        let input = QuoteT::from(input);
        do_derive_type_script_ify(input).into()
    } else {
        proc_macro::TokenStream::new()
    }
}

fn do_derive_typescript_definition(input: QuoteT) -> QuoteT {
    let verify = if cfg!(feature = "type-guards") {
        true
    } else {
        false
    };
    let parsed = Typescriptify::parse(verify, input);
    let export_string = parsed.wasm_string();
    let name = parsed.ctxt.ident.to_string().to_uppercase();

    let export_ident = ident_from_str(&format!("TS_EXPORT_{}", name));

    let mut q = quote! {

        #[wasm_bindgen(typescript_custom_section)]
        pub const #export_ident : &'static str = #export_string;
    };

    if let Some(ref verify) = parsed.wasm_verify() {
        let export_ident = ident_from_str(&format!("TS_EXPORT_VERIFY_{}", name));
        q.extend(quote!(
            #[wasm_bindgen(typescript_custom_section)]
            pub const #export_ident : &'static str = #verify;
        ))
    }

    // just to allow testing... only `--features=test` seems to work
    if cfg!(any(test, feature = "test")) {
        let typescript_ident =
            ident_from_str(&format!("{}___typescript_definition", &parsed.ctxt.ident));

        q.extend(quote!(
            fn #typescript_ident ( ) -> &'static str {
                #export_string
            }

        ));
    }
    if let Some("1") = option_env!("TFY_SHOW_CODE") {
        eprintln!("{}", patch(&q.to_string()));
    }

    q
}

fn do_derive_type_script_ify(input: QuoteT) -> QuoteT {

    let verify = if cfg!(feature = "type-guards") {
        true
    } else {
        false
    };

    let parsed = Typescriptify::parse(verify, input);

    let export_string = parsed.wasm_string();

    // let map = &parsed.map();

    let ident = &parsed.ctxt.ident;


    let (impl_generics, ty_generics, where_clause) = parsed.ctxt.rust_generics.split_for_impl();


    let type_script_guard = if cfg!(feature = "type-guards") {
        let verifier = match parsed.wasm_verify() {
            Some(ref txt) => quote!(Some(::std::borrow::Cow::Borrowed(#txt))),
            None => quote!(None),
        };
        quote!(
            fn type_script_guard() ->  Option<::std::borrow::Cow<'static,str>> {
                    #verifier
            }
        )
    } else {
        quote!()
    };
    let ret = quote! {

        impl #impl_generics ::typescript_definitions::TypeScriptifyTrait for #ident #ty_generics #where_clause {
            fn type_script_ify() ->  ::std::borrow::Cow<'static,str> {
                ::std::borrow::Cow::Borrowed(#export_string)
            }
            #type_script_guard
        }

    };
   
    if let Some("1") = option_env!("TFY_SHOW_CODE") {
        eprintln!("{}", patch(&ret.to_string()));
    }

    ret
}
struct Typescriptify {
    ctxt: ParseContext<'static>,
    body: QuoteMaker,
}
impl Typescriptify {
    fn wasm_string(&self) -> String {
        if self.body.is_enum {
            format!(
                "{}export enum {} {};",
                self.ctxt.global_attrs.to_comment_str(),
                self.ts_ident_str(),
                self.ts_body_str()
            )
        } else {
            format!(
                "{}export type {} = {};",
                self.ctxt.global_attrs.to_comment_str(),
                self.ts_ident_str(),
                self.ts_body_str()
            )
        }
    }
    fn wasm_verify(&self) -> Option<String> {
        match self.body.verify {
            None => None,
            Some(ref body) => {
                let mut s = {
                    let ident = &self.ctxt.ident;
                    let obj = &self.ctxt.arg_name;
                    let body = body.to_string();
                    let body = patch(&body);

                    let generics = self.ts_generics(false);
                    let generics_wb = &generics; // self.ts_generics(true);
                    let is_generic = !self.ctxt.ts_generics.is_empty();
                    let name = guard_name(&ident);
                    if is_generic {
                        format!(
                            "export const {name} = {generics_wb}({obj}: any, typename: string):\
                             {obj} is {ident}{generics} => {body}",
                            name = name,
                            obj = obj,
                            body = body,
                            generics = generics,
                            generics_wb = generics_wb,
                            ident = ident
                        )
                    } else {
                        format!(
                            "export const {name} = {generics_wb}({obj}: any):\
                             {obj} is {ident}{generics} => {body}",
                            name = name,
                            obj = obj,
                            body = body,
                            generics = generics,
                            generics_wb = generics_wb,
                            ident = ident
                        )
                    }
                };
                for txt in self.extra_verify() {
                    s.push('\n');
                    s.push_str(&txt);
                }
                Some(s)
            }
        }
    }
    fn extra_verify(&self) -> Vec<String> {
        let v = self.ctxt.extra.replace(vec![]);
        v.iter()
            .map(|extra| {
                let e = extra.to_string();

                let extra = patch(&e);
                "// generic test  \n".to_string() + &extra
            })
            .collect()
    }

    fn ts_ident_str(&self) -> String {
        let ts_ident = self.ts_ident().to_string();
        patch(&ts_ident).into()
    }
    fn ts_body_str(&self) -> String {
        let ts = self.body.body.to_string();
        let ts = patch(&ts);
        ts.into()
    }
    fn ts_generics(&self, with_bound: bool) -> QuoteT {
        let args_wo_lt: Vec<_> = self.ts_generic_args_wo_lifetimes(with_bound).collect();
        if args_wo_lt.is_empty() {
            quote!()
        } else {
            quote!(<#(#args_wo_lt),*>)
        }
    }
    /// type name suitable for typescript i.e. *no* 'a lifetimes
    fn ts_ident(&self) -> QuoteT {
        let ident = &self.ctxt.ident;
        let generics = self.ts_generics(false);
        quote!(#ident#generics)
    }

    fn ts_generic_args_wo_lifetimes(&self, with_bounds: bool) -> impl Iterator<Item = QuoteT> + '_ {
        self.ctxt.ts_generics.iter().filter_map(move |g| match g {
            Some((ref ident, ref bounds)) => {
                // we ignore trait bounds for typescript
                if bounds.is_empty() || !with_bounds {
                    Some(quote! (#ident))
                } else {
                    let bounds = bounds.iter().map(|ts| &ts.ident);
                    Some(quote! { #ident extends #(#bounds)&* })
                }
            }

            None => None,
        })
    }

    fn parse(gen_verifier: bool, input: QuoteT) -> Self {
        let input: DeriveInput = syn::parse2(input).unwrap();

        let cx = Ctxt::new();
        let mut attrs = attrs::Attrs::new();
        attrs.push_doc_comment(&input.attrs);
        attrs.push_attrs(&input.ident, &input.attrs, Some(&cx));

        let container = ast::Container::from_ast(&cx, &input, Derive::Serialize);
        let ts_generics = ts_generics(container.generics);
        let gv = gen_verifier && attrs.guard;

        let (typescript, ctxt) = {
            let pctxt = ParseContext {
                ctxt: Some(&cx),
                arg_name: quote!(obj),
                global_attrs: attrs,
                gen_guard: gv,
                ident: container.ident.clone(),
                ts_generics: ts_generics,
                rust_generics: container.generics.clone(),
                extra: Cell::new(vec![]),
            };

            let typescript = match container.data {
                ast::Data::Enum(ref variants) => pctxt.derive_enum(variants, &container),
                ast::Data::Struct(style, ref fields) => {
                    pctxt.derive_struct(style, fields, &container)
                }
            };
            // erase serde context
            (
                typescript,
                ParseContext {
                    ctxt: None,
                    ..pctxt
                },
            )
        };


        // consumes context panics with errors
        if let Err(m) = cx.check() {
            panic!(m);
        }
        Self {
            ctxt,
            body: typescript,
        }
    }
}

fn ts_generics(g: &syn::Generics) -> Vec<Option<(Ident, Bounds)>> {
    // lifetime params are represented by None since we are only going
    // to translate them to '_

    // impl#generics TypeScriptTrait for A<... lifetimes to '_ and T without bounds>

    use syn::{ConstParam, GenericParam, LifetimeDef, TypeParam, TypeParamBound};
    g.params
        .iter()
        .map(|p| match p {
            GenericParam::Lifetime(LifetimeDef { /* lifetime,*/ .. }) => None,
            GenericParam::Type(TypeParam { ident, bounds, ..}) => {
                let bounds = bounds.iter()
                    .filter_map(|b| match b {
                        TypeParamBound::Trait(t) => Some(&t.path),
                        _ => None // skip lifetimes for bounds
                    })
                    .map(last_path_element)
                    .filter_map(|b| b)
                    .collect::<Vec<_>>();

                Some((ident.clone(), bounds))
            },
            GenericParam::Const(ConstParam { ident, ty, ..}) => {
                let ty = TSType {
                    ident: ident.clone(),
                    args: vec![ty.clone()],
                    return_type: None,
                };
                Some((ident.clone(), vec![ty]))
            },

        })
        .collect()
}

fn return_type(rt: &syn::ReturnType) -> Option<syn::Type> {
    match rt {
        syn::ReturnType::Default => None, // e.g. undefined
        syn::ReturnType::Type(_, tp) => Some(*tp.clone()),
    }
}

// represents a typescript type T<A,B>
struct TSType {
    ident: syn::Ident,
    args: Vec<syn::Type>,
    return_type: Option<syn::Type>, // only if function
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
                // closures Fn(A,B) -> C
                syn::PathArguments::Parenthesized(syn::ParenthesizedGenericArguments {
                    output,
                    inputs,
                    ..
                }) => {
                    let args: Vec<_> = inputs.iter().cloned().collect();
                    let ret = return_type(output);
                    return Some(TSType {
                        ident,
                        args,
                        return_type: ret,
                    });
                }
                syn::PathArguments::None => {
                    return Some(TSType {
                        ident,
                        args: vec![],
                        return_type: None,
                    });
                }
            };
            // ignore lifetimes
            let args = args
                .iter()
                .filter_map(|p| match p {
                    syn::GenericArgument::Type(t) => Some(t),
                    _ => None, // bindings A=I, expr, constraints A : B ... skip!
                })
                .cloned()
                .collect::<Vec<_>>();

            Some(TSType {
                ident,
                args,
                return_type: None,
            })
        }
        None => None,
    }
}

pub(crate) struct FieldContext<'a> {
    pub ctxt: &'a ParseContext<'a>, //
    pub field: &'a ast::Field<'a>,  // field being parse
    pub attrs: Attrs,               // field attributes
}

impl<'a> FieldContext<'a> {
     pub fn get_path(&self, ty: &syn::Type) -> Option<TSType> {
        use syn::Type::Path;
        use syn::TypePath;
        match ty {
            Path(TypePath { path, .. }) => last_path_element(&path),
            _ => None,
        }
    }   
}

pub(crate) struct ParseContext<'a> {
    ctxt: Option<&'a Ctxt>, // serde parse context for error reporting
    arg_name: QuoteT,       // top level "name" of argument for verifier
    global_attrs: Attrs,    // global #[typescript(...)] attributes
    gen_guard: bool,        // generate type guard for this struct/enum
    ident: syn::Ident,      // name of enum struct
    ts_generics: Vec<Option<(Ident, Bounds)>>, // None means a lifetime parameter
    rust_generics: syn::Generics, // original rust generics
    extra: Cell<Vec<QuoteT>>, // for generic verifier hack!
}


impl<'a> ParseContext<'a> {
    // Some helpers

    fn err_msg(&self, msg: &str) {
        if let Some(ctxt) = self.ctxt {
            ctxt.error(msg);
        } else {
            panic!(msg.to_string())
        }
    }

    fn field_to_ts(&self, field: &ast::Field<'a>) -> QuoteT {
        let attrs = Attrs::from_field(field, self.ctxt);
        // if user has provided a type ... use that
        if attrs.ts_type.is_some() {
            return attrs.ts_type.unwrap();
        }
        let ts = FieldContext {
            attrs,
            ctxt: &self,
            field,
        };
        ts.type_to_ts(&field.ty)
    }

    fn derive_field(&self, field: &ast::Field<'a>) -> QuoteT {
        let field_name = field.attrs.name().serialize_name(); // use serde name instead of field.member
        let field_name = ident_from_str(&field_name);

        let ty = self.field_to_ts(&field);

        quote! {
            #field_name: #ty
        }
    }
    fn derive_fields(
        &'a self,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = QuoteT> + 'a {
        fields.iter().map(move |f| self.derive_field(f))
    }
    fn derive_field_tuple(
        &'a self,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = QuoteT> + 'a {
        fields.iter().map(move |f| self.field_to_ts(f))
    }

    fn check_flatten(&self, fields: &[&'a ast::Field<'a>], ast_container: &ast::Container) -> bool {
        let has_flatten = fields.iter().any(|f| f.attrs.flatten()); // .any(|f| f);
        if has_flatten {
            self.err_msg(&format!(
                    "{}: #[serde(flatten)] does not work for typescript-definitions.",
                    ast_container.ident
                ));
        };
        has_flatten
    }
}
