#![allow(unused)]
use super::patch::TRIPPLE_EQ;
use super::{
    ast, ident_from_str, is_bytes, last_path_element, Attrs, ParseContext, QuoteT, TSType,
};
use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) struct Verify<'a> {
    pub ctxt: &'a ParseContext<'a>,
    pub field: &'a ast::Field<'a>,
    pub attrs: Attrs,
}

impl<'a> Verify<'a> {
    pub fn verify_type(&self, obj: &'a TokenStream, ty: &syn::Type) -> QuoteT {
        // `type_to_ts` recursively calls itself occationally
        // finding a Path which it hands to last_path_element
        // which generates a "simplified" TSType struct which
        // is handed to `generic_to_ts` which possibly "bottoms out"
        // by generating tokens for typescript types.
        let eq = ident_from_str(TRIPPLE_EQ);
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
                    let x = quote!(#obj[#i]);
                    let verify = self.verify_type(&quote!(x), t);
                    quote! {
                        {
                            const x = #x;
                            if (x == undefined) return false;
                            #verify;
                        }
                    }
                });
                let len = elems.len();
                let len = Literal::usize_unsuffixed(len);
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
        if let Some(ty) = self.ctxt.get_path(elem) {
            if ty.ident == "u8" && is_bytes(&self.field) {
                let eq = ident_from_str(TRIPPLE_EQ);
                return quote!(if (! typeof #obj #eq "string") return false);
            };
        };
        let verify = self.verify_type(&quote!(x), elem);
        // TODO: verify first only
        let eq = ident_from_str(TRIPPLE_EQ);
        quote! {
            if (!Array.isArray(#obj)) return false;
            if (#obj.length > 0)
                for (let x of #obj) {
                    #verify;
                }
        }
    }
    fn verify_generic(&self, obj: &'a TokenStream, ts: TSType) -> QuoteT {
        let eq = ident_from_str(TRIPPLE_EQ);
        match ts.ident.to_string().as_ref() {
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
            | "i128" | "isize" | "f64" | "f32" => {
                quote! { if (! typeof #obj #eq "number") return false }
            }
            "String" | "str" => quote! { if (! typeof #obj #eq "string") return false },
            "bool" => quote! { if (! typeof #obj #eq "boolean") return false },
            "Box" | "Cow" | "Rc" | "Arc" if ts.args.len() == 1 => {
                self.verify_type(obj, &ts.args[0])
            }

            // std::collections
            "Vec" | "VecDeque" | "LinkedList" if ts.args.len() == 1 => {
                self.verify_array(obj, &ts.args[0])
            }
            "HashMap" | "BTreeMap" if ts.args.len() == 2 => {
                let k = self.verify_type(&quote!(k), &ts.args[0]);
                let v = self.verify_type(&quote!(v), &ts.args[1]);
                quote!(
                    for (let e of #obj) {
                        let [k, v] = e;
                        #k;
                        #v;
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
                let v = self.verify_type(&v, &ts.args[0]);
                quote! ({
                        if(
                            ((v => {if(v == undefined) return false; #k; return true; })(#obj.Ok)) ||
                            ((v => {if(v == undefined) return false; #v; return true; })(#obj.Err))
                          ) return true;
                        return false;
                 } )
            }
            "Fn" | "FnOnce" | "FnMut" => quote!(),
            _ => {
                let i = ts.ident;
                let func = ident_from_str(&format!("verify_{}", i));
                if !ts.args.is_empty() {
                    // TODO: get type args from to
                    let args = self.ctxt.derive_syn_types(&ts.args, &self.field);
                    quote! { if (!#func<#(#args),*>(#obj)) return false; }
                } else {
                    quote!( if (!#func(#obj)) return false; )
                }
            }
        }
    }
    pub fn verify_field(&self, obj: &TokenStream) -> QuoteT {
        let n = self.field.attrs.name().serialize_name(); // use serde name instead of field.member
        let n = ident_from_str(&n);
        let verify = self.verify_type(&quote!(val), &self.field.ty);

        quote! {
           if (#obj.#n == undefined) return false;
           {
            const val = #obj.#n;
            #verify;
           }
        }
    }
}
impl<'a> ParseContext<'a> {
    pub fn verify_type(&'a self, obj: &'a TokenStream, field: &'a ast::Field<'a>) -> QuoteT {
        let attrs = Attrs::from_field(field, self.ctxt);
        let verify = Verify {
            attrs,
            ctxt: self,
            field,
        };
        verify.verify_type(&obj, &field.ty)
    }
    pub fn verify_fields(
        &'a self,
        obj: &'a TokenStream,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = QuoteT> + 'a {
        fields.iter().map(move |f| {
            let attrs = Attrs::from_field(f, self.ctxt);
            let verify = Verify {
                attrs,
                field: f,
                ctxt: &self,
            };
            verify.verify_field(obj)
        })
    }
    pub fn verify_field_tuple(
        &'a self,
        obj: &'a TokenStream,
        fields: &'a [&'a ast::Field<'a>],
    ) -> impl Iterator<Item = QuoteT> + 'a {
        let eq = ident_from_str(TRIPPLE_EQ);
        fields.iter().enumerate().map(move |(i, f)| {
            let i = Literal::usize_unsuffixed(i);
            let n = quote!(#obj[#i]);
            let attrs = Attrs::from_field(f, self.ctxt);

            let v = Verify {
                attrs,
                field: f,
                ctxt: &self,
            };
            let verify = v.verify_type(&quote!(v), &f.ty);
            quote! {
                if (#n == undefined) return false;
                {
                    const v = #n;
                    #verify;
                }
            }
        })
    }
}
