#![allow(unused)]
use super::{ast, ident_from_str, is_bytes, last_path_element, ParseContext, QuoteT, TSType};
use proc_macro2::TokenStream;
use quote::quote;

impl<'a> ParseContext<'a> {
    fn verify_type(
        &self,
        obj: &'a TokenStream,
        ty: &syn::Type,
        field: &'a ast::Field<'a>,
    ) -> QuoteT {
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
            | Ptr(TypePtr { elem, .. }) => self.verify_array(obj, elem, field),
            Reference(TypeReference { elem, .. }) => self.verify_type(obj, elem, field),
            // fn(a: A,b: B, c:C) -> D
            BareFn(TypeBareFn { output, inputs, .. }) => {
                return quote!();
            }
            Never(..) => quote! { false },
            Tuple(TypeTuple { elems, .. }) => {
                let elems = elems.iter().enumerate().map(|(i, t)| {
                    let x = quote!(#obj[#i]);
                    self.verify_type(&x, t, field)
                });
                quote!(
                    if (! Array.isArray(#obj)) return false;
                    {
                        #(#elems;)*;
                    }
                )
            }

            Path(TypePath { path, .. }) => match last_path_element(&path) {
                Some(ts) => self.verify_generic(obj, ts, field),
                _ => quote! {},
            },
            TraitObject(TypeTraitObject { bounds, .. })
            | ImplTrait(TypeImplTrait { bounds, .. }) => quote!(),
            Paren(TypeParen { elem, .. }) | Group(TypeGroup { elem, .. }) => {
                let verify = self.verify_type(obj, elem, field);
                quote! {  #verify;  }
            }
            Infer(..) | Macro(..) | Verbatim(..) => quote! {},
        }
    }
    fn verify_array(
        &self,
        obj: &'a TokenStream,
        elem: &syn::Type,
        field: &'a ast::Field<'a>,
    ) -> QuoteT {
        if let Some(ty) = self.get_path(elem) {
            if ty.ident == "u8" && is_bytes(field) {
                return quote!(if (! typeof #obj is "string") return false);
            };
        };
        let verify = self.verify_type(&quote!(x), elem, field);
        quote! {
            for (let x of #obj) {
                #verify;
             }
        }
    }
    fn verify_generic(
        &self,
        obj: &'a TokenStream,
        ts: TSType,
        field: &'a ast::Field<'a>,
    ) -> QuoteT {
        match ts.ident.to_string().as_ref() {
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
            | "i128" | "isize" | "f64" | "f32" => {
                quote! { if (! typeof #obj === "number") return false }
            }
            "String" | "str" => quote! { if (! typeof #obj === "string") return false },
            "bool" => quote! { if (! typeof #obj === "boolean") return false },
            "Box" | "Cow" | "Rc" | "Arc" if ts.args.len() == 1 => {
                self.verify_type(obj, &ts.args[0], field)
            }

            // std::collections
            "Vec" | "VecDeque" | "LinkedList" if ts.args.len() == 1 => {
                self.verify_array(obj, &ts.args[0], field)
            }
            "HashMap" | "BTreeMap" if ts.args.len() == 2 => {
                let k = self.verify_type(&quote!(k), &ts.args[0], field);
                let v = self.verify_type(&quote!(v), &ts.args[0], field);
                quote!(
                    for (let e of #obj) {
                        let [k, v] = e;
                        #k;
                        #v;
                    }
                )
            }
            "HashSet" | "BTreeSet" if ts.args.len() == 1 => {
                self.verify_array(obj, &ts.args[0], field)
            }
            "Option" if ts.args.len() == 1 => {
                let verify = self.verify_type(obj, &ts.args[0], field);
                quote!(  if (#obj !== null) {
                            #verify;
                        }
                )
            }
            "Result" if ts.args.len() == 2 => {
                let v = quote!(v);
                let k = self.verify_type(&v, &ts.args[0], field);
                let v = self.verify_type(&v, &ts.args[0], field);
                quote! (
                    if( !(v =>
                        if(
                            (v => {if(v  === undefined) return false; #k; return true; })(v.Ok) ||
                            (v => {if(v === undefined) return false; #v; return true; })(v.Err)
                          ) return true;
                        return false;
                        )(#obj)

                        ) return false
                )
            }
            "Fn" | "FnOnce" | "FnMut" => quote!(),
            _ => {
                let i = ts.ident;
                if !ts.args.is_empty() {
                    // TODO: get type args from to
                    let args = self.derive_syn_types(&ts.args, field);
                    quote! { if (!verify_#i<#(#args),*>(#obj)) return false; }
                } else {
                    quote!( if (!verify_#i(#obj)) return false; )
                }
            }
        }
    }
    fn verify_field(&self, obj: &TokenStream, field: &ast::Field<'a>) -> QuoteT {
        let n = field.attrs.name().serialize_name(); // use serde name instead of field.member
        let n = ident_from_str(&n);

        let verify = self.verify_type(&quote!(v), &field.ty, &field);

        quote! {
           if (#obj.#n === undefined) return false;
           {
            let v = #obj.#n;
            #verify;
           }
        }
    }
    fn verify_fields(
        &'a self,
        obj: &TokenStream,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = QuoteT> + 'a {
        fields.iter().map(move |f| self.derive_field(f))
    }
    fn verify_field_tuple(
        &'a self,
        obj: TokenStream,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = QuoteT> + 'a {
        fields.iter().enumerate().map(move |(i, f)| {
            let n = quote!(#obj[i]);
            self.verify_type(&n, &f.ty, f)
        })
    }
}
