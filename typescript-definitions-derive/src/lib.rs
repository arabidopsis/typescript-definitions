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
use syn::DeriveInput;

mod attrs;
mod derive_enum;
mod derive_struct;
mod patch;
// mod quotet;
mod tests;
mod utils;
mod verify;

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
    let parsed = Typescriptify::parse(false, input);
    let export_string = parsed.wasm_string();
    let name = parsed.ident.to_string().to_uppercase();

    let export_ident = ident_from_str(&format!("TS_EXPORT_{}", name));

    // eprintln!(
    //     "....[typescript] export type {}={};",
    //     parsed.ident, typescript_string
    // );
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
            ident_from_str(&format!("{}___typescript_definition", &parsed.ident));

        q.extend(quote!(
            fn #typescript_ident ( ) -> &'static str {
                #export_string
            }

        ));
    }

    q
}

fn do_derive_type_script_ify(input: QuoteT) -> QuoteT {
    let parsed = Typescriptify::parse(true, input);

    let export_string = parsed.wasm_string();

    // let map = &parsed.map();

    let ident = &parsed.ident;

    let ret = if parsed.ts_generics.is_empty() {
        quote! {

            impl ::typescript_definitions::TypeScriptifyTrait for #ident {
                fn type_script_ify() ->  String {
                    #export_string.into()
                }

                // fn type_script_fields() -> Option<Vec<&'static str>> {
                //     #map
                // }
            }
        }
    } else {
        let generics = parsed.generic_args_with_lifetimes();
        let rustg = &parsed.rust_generics;
        quote! {

            impl#rustg ::typescript_definitions::TypeScriptifyTrait for #ident<#(#generics),*> {
                fn type_script_ify() ->  String {
                    #export_string.into()
                }

                // fn type_script_fields() -> Option<Vec<&'static str>> {
                //     #map
                // }
            }
        }
    };
    if let Some("1") = option_env!("TFY_SHOW_CODE") {
        eprintln!("{}", patch(&ret.to_string()));
    }

    ret
}
struct Typescriptify {
    ctxt: ParseContext<'static>,
    ident: syn::Ident,                         // name of enum struct
    ts_generics: Vec<Option<(Ident, Bounds)>>, // None means a lifetime parameter
    body: QuoteMaker,
    rust_generics: syn::Generics, // original rust generics
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
                let ident = &self.ident;
                let obj = &self.ctxt.verify;
                let body = body.to_string();
                let body = patch(&body);
                let generics = self.ts_generics();
                Some(format!("export const verify_{ident} = {generics}({obj}: any): {obj} is {ident}{generics} => {body}", 
                    ident=ident, obj=obj, body=body, generics=generics ))
            }
        }
    }

    fn ts_ident_str(&self) -> String {
        let ts_ident = self.ts_ident().to_string();
        patch(&ts_ident).into()
    }
    fn ts_body_str(&self) -> String {
        let ts = self.body.body.to_string();
        let ts = patch(&ts);
        return ts.into();
    }
    fn ts_generics(&self) -> QuoteT {
        let args_wo_lt: Vec<_> = self.ts_generic_args_wo_lifetimes(false).collect();
        if args_wo_lt.is_empty() {
            quote!()
        } else {
            quote!(<#(#args_wo_lt),*>)
        }
    }
    /// type name suitable for typescript i.e. *no* 'a lifetimes
    fn ts_ident(&self) -> QuoteT {
        let ident = &self.ident;
        let generics = self.ts_generics();
        quote!(#ident#generics)
    }

    fn ts_generic_args_wo_lifetimes(&self, with_bounds: bool) -> impl Iterator<Item = QuoteT> + '_ {
        self.ts_generics.iter().filter_map(move |g| match g {
            Some((ref ident, ref bounds)) => {
                // we ignore trait bounds for typescript
                if bounds.is_empty() || !with_bounds {
                    Some(quote! (#ident))
                } else {
                    let bounds = bounds.iter().map(|ts| &ts.ident);
                    Some(quote! { #ident extends #(#bounds)&* })
                }
            }

            _ => None,
        })
    }

    fn generic_args_with_lifetimes(&self) -> impl Iterator<Item = QuoteT> + '_ {
        // suitable for impl<...> Trait for T<#generic_args_with_lifetime> ...
        // we need to return quotes because '_ is not an Ident
        self.ts_generics.iter().map(|g| match g {
            Some((ref i, ref _bounds)) => quote!(#i),
            None => quote!('_), // only need '_
        })
    }

    fn parse(is_type_script_ify: bool, input: QuoteT) -> Self {
        let input: DeriveInput = syn::parse2(input).unwrap();

        let cx = Ctxt::new();
        let mut attrs = attrs::Attrs::new();
        attrs.push_doc_comment(&input.attrs);
        attrs.push_attrs(&input.ident, &input.attrs, Some(&cx));

        let container = ast::Container::from_ast(&cx, &input, Derive::Serialize);

        let (typescript, ctxt) = {
            let pctxt = ParseContext::new(is_type_script_ify, attrs, &cx);

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

        let ts_generics = ts_generics(container.generics);

        if false
            && is_type_script_ify
            && ctxt.global_attrs.turbo_fish.is_none()
            && ts_generics.len() > 0
            && ts_generics.iter().any(|f| f.is_some())
        {
            cx.error(format!(
                "Generic item \"{}\" requires #[typescript(turbo_fish= \"...\")] attribute",
                container.ident
            ))
        }

        // consumes context panics with errors
        if let Err(m) = cx.check() {
            panic!(m);
        }
        Self {
            ctxt,
            ident: container.ident,
            ts_generics,
            body: typescript,
            rust_generics: container.generics.clone(), // keep original type generics around for type_script_ify
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

pub(crate) struct ParseContext<'a> {
    ctxt: Option<&'a Ctxt>, // serde parse context for error reporting

    #[allow(unused)]
    is_type_script_ify: bool,
    #[allow(unused)]
    verify: QuoteT,
    global_attrs: Attrs,
}
impl<'a> ParseContext<'a> {
    fn new(is_type_script_ify: bool, global_attrs: Attrs, ctxt: &'a Ctxt) -> ParseContext<'a> {
        ParseContext {
            ctxt: Some(ctxt),
            is_type_script_ify,
            verify: quote!(obj),
            global_attrs,
        }
    }
    fn generic_to_ts(&self, ts: TSType, field: &'a ast::Field<'a>) -> QuoteT {
        let to_ts = |ty: &syn::Type| self.type_to_ts(ty, field);

        match ts.ident.to_string().as_ref() {
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
            | "i128" | "isize" | "f64" | "f32" => quote! { number },
            "String" | "str" => quote! { string },
            "bool" => quote! { boolean },
            "Box" | "Cow" | "Rc" | "Arc" if ts.args.len() == 1 => to_ts(&ts.args[0]),

            // std::collections
            "Vec" | "VecDeque" | "LinkedList" if ts.args.len() == 1 => {
                self.type_to_array(&ts.args[0], field)
            }
            "HashMap" | "BTreeMap" if ts.args.len() == 2 => {
                let k = to_ts(&ts.args[0]);
                let v = to_ts(&ts.args[1]);
                // quote!(Map<#k,#v>)
                quote!( { [key: #k]:#v } )
            }
            "HashSet" | "BTreeSet" if ts.args.len() == 1 => {
                let k = to_ts(&ts.args[0]);
                //quote!(Set<#k>)
                quote! ( #k[] )
            }
            "Option" if ts.args.len() == 1 => {
                let k = to_ts(&ts.args[0]);
                quote!(  #k | null  )
            }
            "Result" if ts.args.len() == 2 => {
                let k = to_ts(&ts.args[0]);
                let v = to_ts(&ts.args[1]);
                // ugh!
                // see patch.rs...
                let vertical_bar = ident_from_str(patch::PATCH);
                quote!(  { Ok : #k } #vertical_bar { Err : #v }  )
            }
            "Fn" | "FnOnce" | "FnMut" => {
                let args = self.derive_syn_types(&ts.args, field);
                if let Some(ref rt) = ts.return_type {
                    let rt = to_ts(rt);
                    quote! { (#(#args),*) => #rt }
                } else {
                    quote! { (#(#args),*) => undefined }
                }
            }
            _ => {
                let ident = ts.ident;
                if !ts.args.is_empty() {
                    // let args = ts.args.iter().map(|ty| self.type_to_ts(ty));
                    let args = self.derive_syn_types(&ts.args, field);
                    quote! { #ident<#(#args),*> }
                } else {
                    quote! {#ident}
                }
            }
        }
    }

    fn get_path(&self, ty: &syn::Type) -> Option<TSType> {
        use syn::Type::Path;
        use syn::TypePath;
        match ty {
            Path(TypePath { path, .. }) => last_path_element(&path),
            _ => None,
        }
    }
    fn type_to_array(&self, elem: &syn::Type, field: &'a ast::Field<'a>) -> QuoteT {
        // check for [u8] or Vec<u8>

        if let Some(ty) = self.get_path(elem) {
            if ty.ident == "u8" && is_bytes(field) {
                return quote!(string);
            };
        };

        let tp = self.type_to_ts(elem, field);
        quote! { #tp[] }
    }
    /// # convert a `syn::Type` rust type to a
    /// `TokenStream` of typescript type: basically i32 => number etc.
    ///
    /// field is the current Field for which we are trying a conversion
    fn type_to_ts(&self, ty: &syn::Type, field: &'a ast::Field<'a>) -> QuoteT {
        // `type_to_ts` recursively calls itself occationally
        // finding a Path which it hands to last_path_element
        // which generates a "simplified" TSType struct which
        // is handed to `generic_to_ts` which possibly "bottoms out"
        // by generating tokens for typescript types.

        use syn::Type::*;
        use syn::{
            BareFnArgName, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeParamBound,
            TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
        };
        match ty {
            Slice(TypeSlice { elem, .. })
            | Array(TypeArray { elem, .. })
            | Ptr(TypePtr { elem, .. }) => self.type_to_array(elem, field),
            Reference(TypeReference { elem, .. }) => self.type_to_ts(elem, field),
            // fn(a: A,b: B, c:C) -> D
            BareFn(TypeBareFn { output, inputs, .. }) => {
                let mut args: Vec<Ident> = Vec::with_capacity(inputs.len());
                let mut typs: Vec<&syn::Type> = Vec::with_capacity(inputs.len());

                for (idx, t) in inputs.iter().enumerate() {
                    let i = match t.name {
                        Some((ref n, _)) => match n {
                            BareFnArgName::Named(m) => m.clone(),
                            _ => ident_from_str("_"), // Wild token '_'
                        },
                        _ => ident_from_str(&format!("_dummy{}", idx)),
                    };
                    args.push(i);
                    typs.push(&t.ty); // TODO: check type is known
                }
                // typescript lambda (a: A, b:B) => C

                // let typs = typs.iter().map(|ty| self.type_to_ts(ty));
                let typs = self.derive_syn_types_ptr(&typs, field);
                if let Some(ref rt) = return_type(&output) {
                    let rt = self.type_to_ts(rt, field);
                    quote! { ( #(#args: #typs),* ) => #rt }
                } else {
                    quote! { ( #(#args: #typs),* ) => undefined}
                }
            }
            Never(..) => quote! { never },
            Tuple(TypeTuple { elems, .. }) => {
                let elems = elems.iter().map(|t| self.type_to_ts(t, field));
                quote!([ #(#elems),* ])
            }

            Path(TypePath { path, .. }) => match last_path_element(&path) {
                Some(ts) => self.generic_to_ts(ts, field),
                _ => quote! { any },
            },
            TraitObject(TypeTraitObject { bounds, .. })
            | ImplTrait(TypeImplTrait { bounds, .. }) => {
                let elems = bounds
                    .iter()
                    .filter_map(|t| match t {
                        TypeParamBound::Trait(t) => last_path_element(&t.path),
                        _ => None, // skip lifetime etc.
                    })
                    .map(|t| self.generic_to_ts(t, field));

                // TODO check for zero length?
                // A + B + C => A & B & C
                quote!(#(#elems)&*)
            }
            Paren(TypeParen { elem, .. }) | Group(TypeGroup { elem, .. }) => {
                let tp = self.type_to_ts(elem, field);
                quote! { ( #tp ) }
            }
            Infer(..) | Macro(..) | Verbatim(..) => quote! { any },
        }
    }

    // Some helpers

    fn field_to_ts(&self, field: &ast::Field<'a>) -> QuoteT {
        self.type_to_ts(&field.ty, field)
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
    fn derive_syn_types_ptr(
        &'a self,
        types: &'a [&'a syn::Type],
        field: &'a ast::Field<'a>,
    ) -> impl Iterator<Item = QuoteT> + 'a {
        types.iter().map(move |ty| self.type_to_ts(ty, field))
    }
    fn derive_syn_types(
        &'a self,
        types: &'a [syn::Type],
        field: &'a ast::Field<'a>,
    ) -> impl Iterator<Item = QuoteT> + 'a {
        types.iter().map(move |ty| self.type_to_ts(ty, field))
    }

    fn check_flatten(&self, fields: &[&'a ast::Field<'a>], ast_container: &ast::Container) -> bool {
        let has_flatten = fields.iter().any(|f| f.attrs.flatten()); // .any(|f| f);
        if has_flatten {
            if let Some(ref ct) = self.ctxt {
                ct.error(format!(
                    "{}: #[serde(flatten)] does not work for typescript-definitions.",
                    ast_container.ident
                ));
            }
        };
        has_flatten
    }
}
