#![allow(unused)]
extern crate serde_derive_internals;
use quote::ToTokens;
use proc_macro2::TokenStream;
use serde_derive_internals::{ast, attr, attr::EnumTag};
use super::{QuoteT, ident_from_str, type_to_ts, TagInfo};

struct Pair {
    key: String,
    value: Box<dyn BuilderTrait>,
}

impl Pair {
    fn new<'a>(field: &ast::Field<'a>) -> Pair {
        let field_name = field.attrs.name().serialize_name();
        let ty = type_to_ts(&field.ty, 0);
        Pair {key: field_name, value: ty}
    }
}


struct StructBuilder {
    attrs : Vec<Pair>,
}

struct TupleStructBuilder {
    attrs: Vec<Box<dyn BuilderTrait>>,
}
struct StructVariantBuilder  {
    variant_name: String,
    taginfo : TagInfo,
    attrs : Vec<Pair>,
}
struct TupleVariantBuilder  {
    variant_name: String,
    taginfo : TagInfo,
    attrs: Vec<Box<dyn BuilderTrait>>,
}

struct EnumBuilder  {
    attrs: Vec<Box<dyn BuilderTrait>>,
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
            .map(|field| type_to_ts(field.ty, 0));
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
            .map(|field| type_to_ts(field.ty, 0));
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

pub trait BuilderTrait : ToTokens  {
    fn ts(&self) -> QuoteT;
}

impl BuilderTrait for QuoteT {
    fn ts(&self) -> QuoteT {
        self.clone()
    }
}

fn vecpair2quote(v : &Vec<Pair>) -> QuoteT {
       //  let name = ident_from_str(&self.name);
        let k = v.iter().map(|p| ident_from_str(&p.key));
        let v = v.iter().map(|p| p.value.ts());
        quote! (  #(#k : #v),*  )
}
fn vec2quote(v : &Vec<Box<BuilderTrait>>) -> QuoteT {
        let n = v.len();
        let v = v.iter().map(|p| p.ts());
        if n == 0 {
            quote!( {} )
        } else if n == 1 {
            quote! ( #(#v),* )
        } else {
            quote! ( [ #(#v),* ] )
        }
}

impl BuilderTrait for StructBuilder {
    fn ts(&self)  -> QuoteT {
        let v = vecpair2quote(&self.attrs);
        quote!( { #v } )
    }

}

impl ToTokens for StructBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ts().to_tokens(tokens)
    }
}

impl BuilderTrait for TupleStructBuilder {
    fn ts(&self)  -> QuoteT {
        vec2quote(&self.attrs)
    }
}


impl ToTokens for TupleStructBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ts().to_tokens(tokens)
    }
} 

impl BuilderTrait for StructVariantBuilder {

    fn ts(&self) -> QuoteT {
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
impl ToTokens for StructVariantBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ts().to_tokens(tokens)
    }
} 

impl BuilderTrait for TupleVariantBuilder {
    fn ts(&self)  -> QuoteT {
        // let name = ident_from_str(&self.name);
        let v = self.attrs.iter().map(|p| p.ts());
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
impl ToTokens for TupleVariantBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ts().to_tokens(tokens)
    }
} 
impl BuilderTrait for EnumBuilder {
    fn ts(&self) -> QuoteT {
        let v = self.attrs.iter().map(|p| p.ts());
        quote! { #(#v)|* }
    }
}
impl ToTokens for EnumBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ts().to_tokens(tokens)
    }
} 
pub(crate) fn derive_struct<'a>(
    style: ast::Style,
    fields: &[ast::Field<'a>],
    attr_container: &attr::Container,
) -> Box<dyn BuilderTrait>  {
    match style {
        ast::Style::Struct =>  Box::new(StructBuilder::new(fields, attr_container)),
        ast::Style::Newtype | ast::Style::Tuple =>
                Box::new(TupleStructBuilder::new(fields, attr_container)),
        ast::Style::Unit => Box::new(TupleStructBuilder::new(&[], attr_container)),
    }
}

pub(crate) fn derive_enum<'a>(variants: &[ast::Variant<'a>], attrs: &attr::Container) -> Box<dyn BuilderTrait> {
    Box::new(EnumBuilder::new(variants, attrs))
}

pub(crate) fn q2b(q: QuoteT) -> Box<dyn BuilderTrait> {
    Box::new(q)
}
