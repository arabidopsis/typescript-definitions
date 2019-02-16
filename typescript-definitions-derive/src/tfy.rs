use super::{filter_visible, ident_from_str, ParseContext, QuoteMaker};

impl<'a> ParseContext<'_> { 
    fn type_to_value(&self, ty: &syn::Type, field: &'a ast::Field<'a>) -> QuoteT {

        use syn::Type::*;
        use syn::{
            BareFnArgName, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeParamBound,
            TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
        };
        match ty {
            Slice(TypeSlice { elem, .. })
            | Array(TypeArray { elem, .. })
            | Ptr(TypePtr { elem, .. }) => self.type_to_array_value(elem, field),
            Reference(TypeReference { elem, .. }) => self.type_to_value(elem, field),
            // fn(a: A,b: B, c:C) -> D
            BareFn(TypeBareFn { output, inputs, .. }) => {
                panic!("can't create a function value")
            }
            Never(..) => quote! { never },
            Tuple(TypeTuple { elems, .. }) => {
                let elems = elems.iter().map(|t| self.type_to_value(t, field));
                quote!(( #(#elems),* ))
            }

            Path(TypePath { path, .. }) => match last_path_element(&path) {
                Some(ts) => self.generic_to_value(ts, field),
                _ => quote! panic!("can't create value for {}", quote!(#path)),
            },
            TraitObject(TypeTraitObject { bounds, .. })
            | ImplTrait(TypeImplTrait { bounds, .. }) => {
                panic!("can't create value for a trait")
            }
            Paren(TypeParen { elem, .. }) | Group(TypeGroup { elem, .. }) => {
                let tp = self.type_to_value(elem, field);
                quote! { ( #tp ) }
            }
            ret i @ Infer(..) | ref i @ Macro(..) | ref i @ Verbatim(..) => 
                panic!("can't create value for {}", quote!(#))
        }
    }
    fn type_to_array_value(&self, elem: &syn::Type, field: &'a ast::Field<'a>) -> QuoteT {

        let val = self.type_to_value(elem, field);
        quote! { vec![#val] }
    }
}