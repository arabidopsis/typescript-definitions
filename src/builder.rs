#![allow(unused)]
extern crate serde_derive_internals;
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::Ident;
use serde_derive_internals::{ast, attr, attr::EnumTag};
use super::{QuoteT, ident_from_str};

type DynBuilder = Box<dyn BuilderTrait>;

#[derive(Clone)]
struct TagInfo {
    tag: String,
    content: Option<String>,
}
fn type_to_ts(ty: &syn::Type) -> DynBuilder {

}

#[derive(Clone)]
struct Pair {
    key: String,
    value: DynBuilder,
}

impl Pair {
    fn new<'a>(field: &ast::Field<'a>) -> Pair {
        let field_name = field.attrs.name().serialize_name();
        let ty = type_to_ts(&field.ty);
        Pair {key: field_name, value: ty}
    }
}


#[derive(Clone)]
struct StructBuilder {
    attrs : Vec<Pair>,
}

struct TupleStructBuilder {
    attrs: Vec<DynBuilder>,
}
struct StructVariantBuilder  {
    variant_name: String,
    taginfo : TagInfo,
    attrs : Vec<Pair>,
}
struct TupleVariantBuilder  {
    variant_name: String,
    taginfo : TagInfo,
    attrs: Vec<DynBuilder>,
}

struct EnumBuilder  {
    attrs: Vec<DynBuilder>,
}

pub struct RefBuilder {
    name: Ident,
    attrs: Vec<DynBuilder>,
}

impl StructBuilder {
    
    fn new<'a>(fields: &[ast::Field<'a>], ct: &attr::Container,) -> StructBuilder {
        let content = fields.into_iter()
            .map(|field| Pair::new(&field));
        // let name = ct.name().serialize_name();
        StructBuilder {attrs : content.collect::<Vec<_>>()}
    }
}
impl TupleStructBuilder {
    
    fn new<'a>(fields: &[ast::Field<'a>], ct: &attr::Container,) -> TupleStructBuilder {
        let content = fields.into_iter()
            .map(|field| type_to_ts(field.ty));
            //.map(|q| Box::new(q) as Box<BuilderTrait>);
        // let name = ct.name().serialize_name();
        TupleStructBuilder { attrs : content.collect::<Vec<_>>()}
    }
}

impl StructVariantBuilder {
    fn new<'a>(
    taginfo: &TagInfo,
    variant_name: &str,
    fields: &[ast::Field<'a>],
        ) -> StructVariantBuilder {
        let content = fields.into_iter()
            .map(|field| Pair::new(&field));

        StructVariantBuilder {variant_name : variant_name.into(), 
            taginfo: taginfo.clone(), attrs: content.collect::<Vec<_>>()}
    }
}
impl TupleVariantBuilder {
    fn new<'a>(
    taginfo: &TagInfo,
    variant_name: &str,
    fields: &[ast::Field<'a>],
        ) -> TupleVariantBuilder {
        let content = fields.into_iter()
            .map(|field| type_to_ts(field.ty));
            //.map(|q| Box::new(q) as Box<BuilderTrait>);

        TupleVariantBuilder {variant_name : variant_name.into(), 
            taginfo: taginfo.clone(), attrs: content.collect::<Vec<_>>()}
    }
}

impl EnumBuilder {
    fn new<'a>(variants: &[ast::Variant<'a>], attrs: &attr::Container) -> EnumBuilder {
        // let n = variants.len() - 1;
        let taginfo = match attrs.tag() {
            EnumTag::Internal { tag, .. } => TagInfo { tag: tag.clone(), content: None },
            EnumTag::Adjacent { tag, content, .. } => TagInfo {
                tag : tag.clone(),
                content: Some(content.clone()),
            },
            _ => TagInfo {
                tag: "kind".into(),
                content: None,
            },
        };
        let content = variants.into_iter().map(|variant| {
            let variant_name = variant.attrs.name().serialize_name();
            match variant.style {
                ast::Style::Struct => {
                    let b = Box::new(StructVariantBuilder::new(&taginfo, &variant_name, &variant.fields));
                    b as Box<BuilderTrait>
                },

                ast::Style::Newtype | ast::Style::Tuple | ast::Style::Unit => {
                    let b = Box::new(TupleVariantBuilder::new(&taginfo, &variant_name, &variant.fields));
                    b as Box<BuilderTrait>
                }
            }
        });
        EnumBuilder { attrs: content.collect::<Vec<_>>()}
    }
}

impl RefBuilder {
    pub fn new(name: Ident, attrs: Vec<DynBuilder>) -> RefBuilder {
        RefBuilder { name, attrs}
    }
    pub fn boxed(name: Ident, attrs: Vec<DynBuilder>) -> DynBuilder {
        Box::new(Self::new(name, attrs))
    }
}

pub trait BuilderTrait : ToTokens {
    fn to_typescript(&self) -> QuoteT;
    fn boxed(&self) -> DynBuilder;
}

impl BuilderTrait for QuoteT {
    fn to_typescript(&self) -> QuoteT {
        self.clone()
    }
    fn boxed(&self) -> DynBuilder {
        Box::new(self) as DynBuilder
    }
}

fn vecpair2quote(v : &Vec<Pair>) -> QuoteT {
       //  let name = ident_from_str(&self.name);
        let k = v.iter().map(|p| ident_from_str(&p.key));
        let v = v.iter().map(|p| &p.value);
        quote! (  #(#k : #v),*  )
}
fn vec2quote(v : &Vec<DynBuilder>) -> QuoteT {
        let n = v.len();
        if n == 0 {
            quote!( {} )
        } else if n == 1 {
            quote! ( #(#v),* )
        } else {
            quote! ( [ #(#v),* ] )
        }
}

impl BuilderTrait for StructBuilder {
    fn to_typescript(&self)  -> QuoteT {
        let v = vecpair2quote(&self.attrs);
        quote!( { #v } )
    }
    fn boxed(&self) -> DynBuilder {
        Box::new(self.clone()) as DynBuilder
    }

}

impl BuilderTrait for TupleStructBuilder {
    fn to_typescript(&self)  -> QuoteT {
        vec2quote(&self.attrs)
    }
}
impl BuilderTrait for RefBuilder {
    fn to_typescript(&self)  -> QuoteT {
        let n = &self.name;
        let v = &self.attrs;
        if v.len() == 0 {
            quote! (#n)
        } else {
            quote!( #n<#(#v),*> )
        }
    }
}


impl BuilderTrait for StructVariantBuilder {

    fn to_typescript(&self) -> QuoteT {
        let tag = ident_from_str(&self.taginfo.tag);
        let n = self.attrs.len();
        let variant_name = &self.variant_name;
        if n == 0 {
            return quote! { # tag: # variant_name }
        }
        let contents = vecpair2quote(&self.attrs);
        if let Some(ref content) = self.taginfo.content {
            let content = ident_from_str(content);
            quote! {
                { #tag: #variant_name, #content: { #contents } }
            }
        } else {
            quote! {
                { #tag: #variant_name, #contents }
            }
        }
    }
}


impl BuilderTrait for TupleVariantBuilder {
    fn to_typescript(&self)  -> QuoteT {
        // let name = ident_from_str(&self.name);
        let v = &self.attrs;
        let n = self.attrs.len();
        let tag = ident_from_str(&self.taginfo.tag);
        let variant_name = &self.variant_name;
        let content = if let Some(ref content) = self.taginfo.content {
            ident_from_str(content)
        } else {
            ident_from_str("fields")
       
        };
    
        if n == 0 {
            quote!( { #tag: #variant_name } )
        } else if n == 1  {
            quote! ( { #tag: #variant_name, #content:  #(#v),* } )
        } else {
            quote! ( { #tag: #variant_name, #content : [ #(#v),* ] } )
        }
    }
}
 
impl BuilderTrait for EnumBuilder {
    fn to_typescript(&self) -> QuoteT {
        let v = &self.attrs;
        quote! { #(#v)|* }
    }
}
// because there is a blanket impl of ToToken for &'a T for alll T : ToTokens
// impl<T: BuilderTrait> ToTokens for T where T: Sized, T: ?AsRef { 
//     fn to_tokens(&self, tokens: &mut TokenStream) { 
//         self.to_typescript().to_tokens(tokens)
//     }
// }
// can't seem to do blanket implementatioon of ToTokens e.g. 

macro_rules! totokens {
    ($($n:ident)*)  => {    
        $(
            impl ToTokens for $n {
                fn to_tokens(&self, tokens: &mut TokenStream) {
                    self.to_typescript().to_tokens(tokens)
                }
            }

        )* 
    }
}

totokens!(RefBuilder StructBuilder TupleStructBuilder StructVariantBuilder TupleVariantBuilder EnumBuilder);


pub(crate) fn derive_struct<'a>(
    style: ast::Style,
    fields: &[ast::Field<'a>],
    attr_container: &attr::Container,
) -> DynBuilder  {
    match style {
        ast::Style::Struct =>  Box::new(StructBuilder::new(fields, attr_container)),
        ast::Style::Newtype | ast::Style::Tuple =>
                Box::new(TupleStructBuilder::new(fields, attr_container)),
        ast::Style::Unit => Box::new(TupleStructBuilder::new(&[], attr_container)),
    }
}

pub(crate) fn derive_enum<'a>(variants: &[ast::Variant<'a>], attrs: &attr::Container) -> DynBuilder {
    Box::new(EnumBuilder::new(variants, attrs))
}

pub(crate) fn q2b(q: QuoteT) -> DynBuilder {
    Box::new(q)
}
