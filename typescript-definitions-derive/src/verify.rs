// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(unused)]

use super::{
    ast, guard_name, ident_from_str, is_bytes, last_path_element, patch::eq, patch::patch, Attrs,
    FieldContext, ParseContext, QuoteT, TSType,
};
use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::quote;

impl<'a> FieldContext<'a> {
    fn verify_type(&self, obj: &'a TokenStream, ty: &syn::Type) -> QuoteT {
        // obj is an Ident
        // remeber obj is definitely *not* undefined... but because
        // of the option type it *could* be null....
        let eq = eq();
        use syn::Type::*;
        use syn::{
            BareFnArgName, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeParamBound,
            TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
        };
        match ty {
            Slice(TypeSlice { elem, .. })
            | Array(TypeArray { elem, .. })
            | Ptr(TypePtr { elem, .. }) => self.verify_array(obj, elem),
            Reference(TypeReference { elem, .. }) => self.verify_type(obj, elem),
            // fn(a: A,b: B, c:C) -> D
            BareFn(TypeBareFn { output, inputs, .. }) => {
                return quote!(); // can you type check functions?
            }
            Never(..) => quote! { false },
            Tuple(TypeTuple { elems, .. }) => {
                let elems = elems.iter().enumerate().map(|(i, t)| {
                    let i = Literal::usize_unsuffixed(i);
                    let v = quote!(#obj[#i]);
                    let verify = self.verify_type(&quote!(val), t);
                    quote! {
                        {
                            const val = #v;
                            if (val #eq undefined) return false;
                            #verify;
                        }
                    }
                });

                let len = Literal::usize_unsuffixed(elems.len());
                quote!(
                    if (! Array.isArray(#obj) || ! #obj.length #eq #len ) return false;
                    {
                        #(#elems;)*;
                    }
                )
            }

            Path(TypePath { path, .. }) => match last_path_element(&path) {
                Some(ts) => self.verify_generic(obj, ts),
                _ => quote! {},
            },
            TraitObject(TypeTraitObject { bounds, .. })
            | ImplTrait(TypeImplTrait { bounds, .. }) => quote!(),
            Paren(TypeParen { elem, .. }) | Group(TypeGroup { elem, .. }) => {
                let verify = self.verify_type(obj, elem);
                quote! {  ( #verify; )  }
            }
            Infer(..) | Macro(..) | Verbatim(..) => quote! {},
        }
    }
    fn verify_array(&self, obj: &'a TokenStream, elem: &syn::Type) -> QuoteT {
        if let Some(ty) = self.get_path(elem) {
            if ty.ident == "u8" && is_bytes(&self.field) {
                let eq = eq();
                return quote!(if (! (typeof #obj #eq "string")) return false);
            };
        };
        let verify = self.verify_type(&quote!(x), elem);
        let brk = if self.attrs.only_first {
            quote!(break;)
        } else {
            quote!()
        };

        quote! {
            if (!Array.isArray(#obj)) return false;
                for (let x of #obj) {
                    #verify;
                    #brk
                }
        }
    }
    fn verify_generic(&self, obj: &'a TokenStream, ts: TSType) -> QuoteT {
        let eq = eq();
        match ts.ident.to_string().as_ref() {
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
            | "i128" | "isize" | "f64" | "f32" => {
                quote! { if (! (typeof #obj #eq "number")) return false }
            }
            "String" | "str" => quote! { if (! (typeof #obj #eq "string")) return false },
            "bool" => quote! { if (! (typeof #obj #eq "boolean")) return false },
            "Box" | "Cow" | "Rc" | "Arc" if ts.args.len() == 1 => {
                self.verify_type(obj, &ts.args[0])
            }

            // std::collections
            "Vec" | "VecDeque" | "LinkedList" if ts.args.len() == 1 => {
                self.verify_array(obj, &ts.args[0])
            }
            "HashMap" | "BTreeMap" if ts.args.len() == 2 => {
                // k will always be strings
                // but tsc seems to check against  {[K in number]: T }
                let k = self.type_to_ts(&ts.args[0], &self.field).to_string();
                let k = if k == "number" {
                    quote! {
                        if (+k #eq NaN) return false;
                    }
                } else {
                    //self.verify_type(&quote!(k), &ts.args[0]);
                    // always going to be a string
                    quote!()
                };
                let v = self.verify_type(&quote!(v), &ts.args[1]);
                let brk = if self.attrs.only_first {
                    quote!(break;)
                } else {
                    quote!()
                };
                // obj is definitely not undefined... but it might be null...
                quote!(
                    if (#obj #eq null || !(typeof #obj #eq "object")) return false;
                    for (let k in #obj) {
                        let v = #obj[k];
                        #k;
                        #v;
                        #brk
                    }
                )
            }
            "HashSet" | "BTreeSet" if ts.args.len() == 1 => self.verify_array(obj, &ts.args[0]),
            "Option" if ts.args.len() == 1 => {
                let verify = self.verify_type(obj, &ts.args[0]);
                quote!(  if (!(#obj #eq null)) { // sic! === to null.
                            #verify;
                        }
                )
            }
            "Result" if ts.args.len() == 2 => {
                let v = quote!(v);
                let k = self.verify_type(&v, &ts.args[0]);
                let v = self.verify_type(&v, &ts.args[1]);
                quote! ({
                        if (#obj #eq null) return false;
                        if(
                            ((v => {if(v == undefined) return false; #k; return true; })(#obj.Ok)) ||
                            ((v => {if(v == undefined) return false; #v; return true; })(#obj.Err))
                          ) return true;
                        return false;
                 } )
            }
            "Fn" | "FnOnce" | "FnMut" => quote!(), // skip
            _ => {
                // Here we go.....
                let ident = ts.ident;

                let is_generic = self.ctxt.ts_generics.iter().any(|v| match v {
                    Some((t, _)) => *t == ident,
                    None => false,
                });
                let func = guard_name(&ident);

                let (func, gen_params): (TokenStream, TokenStream) = if is_generic {
                    (quote!(#func), quote!(<#ident>))
                } else {
                    (quote!(#func), quote!())
                };
                if !ts.args.is_empty() {
                    if is_generic {
                        // T<K,V> with T generic ...
                        self.ctxt.err_msg(&format!(
                            "{}: generic args of a generic type is not supported",
                            ident
                        ));
                        return quote!(return false);
                    }
                    let args: Vec<_> = self.derive_syn_types(&ts.args, &self.field).collect();
                    let a = args.clone();
                    let a = quote!(#(#a),*).to_string();
                    let a = a.trim();
                    if !self.attrs.user_type_guard {
                        if (!self.ok_ts_type(a)) {
                            self.ctxt.err_msg(&format!(
                                "{}: only monomorphization of number, string or boolean permitted: got \"{}\"",
                                ident, patch(&a)
                            ));
                            self.ctxt.err_msg("try a user_type_guard");
                            return quote!(return false);
                        };
                    };
                    let a = Literal::string(&a);
                    quote! { if (!#func#gen_params<#(#args),*>(#obj, #a)) return false; }
                } else {
                    if is_generic {
                        if !self.attrs.user_type_guard {
                            let gen_func = quote!(
                                export const #func = #gen_params(#obj: any, typename: string): #obj is #ident => {
                                    return typeof #obj #eq typename
                                }
                            );
                            self.ctxt.add_extra_guard(gen_func);
                        };

                        quote!( if (!#func#gen_params(#obj, typename)) return false; )
                    } else {
                        quote!( if (!#func#gen_params(#obj)) return false; )
                    }
                }
            }
        }
    }
    pub fn verify_field(&self, obj: &TokenStream) -> QuoteT {
        let n = self.field.attrs.name().serialize_name(); // use serde name instead of field.member
        let n = ident_from_str(&n);
        let val = quote!(val);
        let eq = eq();

        let verify = self.verify_single_type(&val);

        quote! {
           if (#obj.#n #eq undefined) return false;
           {
            const val = #obj.#n;
            #verify;
           }
        }
    }
    fn ok_ts_type(&self, a: &str) -> bool {
        (a == "number" || a == "string" || a == "boolean")
    }
    pub fn verify_single_type(&self, obj: &TokenStream) -> QuoteT {
        if let Some(ref tokens) = self.attrs.as_ts {
            let eq = eq();
            let tokens = tokens.to_string();
            if !self.ok_ts_type(&tokens) {
                self.ctxt.err_msg(&format!(
                    "only string, number or boolean permitted: got \"{}\"",
                    tokens
                ));
            }
            quote!( if(!(typeof #obj #eq #tokens)) return false; )
        } else {
            self.verify_type(obj, &self.field.ty)
        }
    }
}

impl<'a> ParseContext<'a> {
    pub fn verify_type(&'a self, obj: &'a TokenStream, field: &'a ast::Field<'a>) -> QuoteT {
        let attrs = Attrs::from_field(field, self.ctxt);
        let verify = FieldContext {
            attrs,
            ctxt: &self,
            field,
        };
        verify.verify_single_type(&obj)
    }
    pub fn verify_field(&'a self, obj: &'a TokenStream, field: &'a ast::Field<'a>) -> QuoteT {
        let attrs = Attrs::from_field(field, self.ctxt);
        let verify = FieldContext {
            attrs,
            field: field,
            ctxt: &self,
        };
        verify.verify_field(obj)
    }
    pub fn verify_fields(
        &'a self,
        obj: &'a TokenStream,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = QuoteT> + 'a {
        fields.iter().map(move |f| self.verify_field(obj, f))
    }
    pub fn verify_field_tuple(
        &'a self,
        obj: &'a TokenStream,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = QuoteT> + 'a {
        let eq = eq();
        fields.iter().enumerate().map(move |(i, f)| {
            let i = Literal::usize_unsuffixed(i);
            let n = quote!(#obj[#i]);
            let verify = self.verify_type(&quote!(val), f);
            quote! {
                if (#n #eq undefined) return false;
                {
                    const val = #n;
                    #verify;
                }
            }
        })
    }

    fn add_extra_guard(&'a self, tokens: QuoteT) {
        let mut v = self.extra.replace(vec![]);
        v.push(tokens);
        self.extra.set(v);
    }
}
