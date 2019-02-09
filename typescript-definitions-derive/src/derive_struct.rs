// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use serde_derive_internals::{ast};
use super::patch;

struct Fields {
    fields : Vec<String>,
    body : Vec<QuoteT>
}
impl Tbuild for Fields {
    fn build(&self) -> QuoteT {
        let content = &self.body;
        let fields = &self.fields;
        let s = quote!({#(#content),*}).to_string();
        let s = patch(&s);
        quote!( {
                let f = vec![#(#fields),*];
                #s
            }
           
        )

    }
    fn map(&self) -> Option<QuoteT> {
        let fields = &self.fields;
        Some(
            quote! {
                Some(vec![#(#fields),*])
            }
        )
    }
}

use super::{filter_visible, QuoteMaker, quotet::Tbuild, QuoteT, ParseContext};
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
        _ast_container: &ast::Container,
    ) -> QuoteMaker {
        if field.attrs.skip_serializing() {
            return self.derive_struct_unit();
        }
        self.type_to_ts(&field.ty).into()
    }

    fn derive_struct_unit(&self) -> QuoteMaker {
        quote!({}).into()
    }


    fn derive_struct_named_fields(
        &self,
        fields: &[ast::Field<'a>],
        _ast_container: &ast::Container,

    ) -> QuoteMaker {
        let fields = filter_visible(fields);
        if fields.len() == 0 {
            return self.derive_struct_unit();
        };
        if self.is_type_script_ify {
            let names = fields.iter().map(|f| f.attrs.name().serialize_name()).collect::<Vec<_>>();
            let content = fields.iter().map(|f| self.derive_field(f)).collect::<Vec<_>>();
            QuoteMaker::from_builder(Fields { fields: names, body: content })
        } else {
            let content = fields.iter().map(|f| self.derive_field(f));
            quote!({#(#content),*}).into()
        }

    }

    fn derive_struct_tuple(
        &self,
        fields: &[ast::Field<'a>],
        _ast_container: &ast::Container,
    ) -> QuoteMaker {
        let fields = filter_visible(fields);
        if fields.len() == 0 {
            return self.derive_struct_unit();
        }
        let content = fields.iter().map(|f| self.type_to_ts(f.ty));
        quote!([#(#content),*]).into()
    }
}
