// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde_derive_internals::ast;
use quote::quote;
use super::{patch, Ident};

#[allow(unused)]
struct Fields {
    fields: Vec<String>,
    body: Vec<QuoteT>,
    flatten: Vec<Ident>,
}
impl Tbuild for Fields {
    fn build(&self) -> QuoteT {
        let content = &self.body;
        let fields = &self.fields;

        let flatten = self
            .flatten
            .iter()
            .map(|i| quote!(let _ = <#i>::type_script_ify()));
        let s = quote!({#(#content),*}).to_string();
        let s = patch(&s);
        quote!( {
                let f = vec![#(#fields),*];
                #(#flatten);*;
                #s
            }

        )
    }
    fn map(&self) -> Option<QuoteT> {
        let fields = &self.fields;
        Some(quote! {
            Some(vec![#(#fields),*])
        })
    }
}

use super::{filter_visible, quotet::Tbuild, ParseContext, QuoteMaker, QuoteT};
impl<'a> ParseContext<'_> {
    pub(crate) fn derive_struct(
        &self,
        style: ast::Style,
        fields: &[ast::Field<'a>],
        container: &ast::Container,
    ) -> QuoteMaker {
        match style {
            ast::Style::Struct => self.derive_struct_named_fields(fields, container),
            ast::Style::Newtype => self.derive_struct_newtype(&fields[0], container),
            ast::Style::Tuple => self.derive_struct_tuple(fields, container),
            ast::Style::Unit => self.derive_struct_unit(),
        }
    }

    fn derive_struct_newtype(
        &self,
        field: &ast::Field<'a>,
        ast_container: &ast::Container,
    ) -> QuoteMaker {
        if field.attrs.skip_serializing() {
            return self.derive_struct_unit();
        }
        self.check_flatten(&[field], ast_container);
        self.field_to_ts(field).into()
    }

    fn derive_struct_unit(&self) -> QuoteMaker {
        quote!({}).into()
    }

    fn derive_struct_named_fields(
        &self,
        fields: &[ast::Field<'a>],
        ast_container: &ast::Container,
    ) -> QuoteMaker {
        let fields = filter_visible(fields);
        if fields.len() == 0 {
            return self.derive_struct_unit();
        };


        if fields.len() == 1 && ast_container.attrs.transparent() {
            return self.derive_struct_newtype(&fields[0], ast_container);
        };
        self.check_flatten(&fields, ast_container);
        let content = self.derive_fields(&fields);
        quote!({#(#content),*}).into()
        /*
              if self.is_type_script_ify {
                   let mut flatten = Vec::new();
                   for field in &fields {
                       if field.attrs.flatten() {
                           if let Some(ts) = self.get_path(&field.ty) {
                               flatten.push(ts.ident.clone());

                           }
                       }
                   };
                   let names = fields.iter().map(|f| f.attrs.name().serialize_name()).collect::<Vec<_>>();
                   let content = content.collect::<Vec<_>>();
                   QuoteMaker::from_builder(Fields { fields: names, body: content, flatten })
               } else {
                   quote!({#(#content),*}).into()
               }
        */
    }

    fn derive_struct_tuple(
        &self,
        fields: &[ast::Field<'a>],
        ast_container: &ast::Container,
    ) -> QuoteMaker {
        let fields = filter_visible(fields);
        if fields.len() == 0 {
            return self.derive_struct_unit();
        }
        self.check_flatten(&fields, ast_container);
        let content = self.derive_field_types(&fields);

        quote!([#(#content),*]).into()
    }
}
