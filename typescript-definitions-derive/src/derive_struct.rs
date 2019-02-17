// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2::Literal;
use quote::quote;
use serde_derive_internals::ast;

use super::{filter_visible, patch::NL_PATCH, ParseContext, QuoteMaker};

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

        let verify = if self.gen_verifier {
            let v = self.verify_type(&self.arg_name, field);
            Some(quote!( { #v; return true; } ))
        } else {
            None
        };

        QuoteMaker {
            body: self.field_to_ts(field),
            verify,
            is_enum: false,
        }
    }

    fn derive_struct_unit(&self) -> QuoteMaker {
        let verify = if self.gen_verifier {
            let obj = &self.arg_name;
            Some(quote!({ if (#obj == undefined) return false; return true; }))
        } else {
            None
        };
        QuoteMaker {
            body: quote!({}),
            verify,
            is_enum: false,
        }
    }

    fn derive_struct_named_fields(
        &self,
        fields: &[ast::Field<'a>],
        ast_container: &ast::Container,
    ) -> QuoteMaker {
        let fields = filter_visible(fields);
        if fields.is_empty() {
            return self.derive_struct_unit();
        };

        if fields.len() == 1 && ast_container.attrs.transparent() {
            return self.derive_struct_newtype(&fields[0], ast_container);
        };
        self.check_flatten(&fields, ast_container);
        let content = self.derive_fields(&fields);

        let verify = if self.gen_verifier {
            let obj = &self.arg_name;
            let v = self.verify_fields(&obj, &fields);
            let n = fields.len();
            let l = Literal::string(NL_PATCH);
            let nl = (0..n).map(|_| quote!(#l));
            Some(quote!( { if (#obj == undefined) return false; #( #nl #v;)* #l return true; } ))
        } else {
            None
        };

        QuoteMaker {
            body: quote!({#(#content);*}),
            verify,
            is_enum: false,
        }
    }

    fn derive_struct_tuple(
        &self,
        fields: &[ast::Field<'a>],
        ast_container: &ast::Container,
    ) -> QuoteMaker {
        let fields = filter_visible(fields);
        if fields.is_empty() {
            return self.derive_struct_unit();
        }
        self.check_flatten(&fields, ast_container);
        let content = self.derive_field_tuple(&fields);
        let verify = if self.gen_verifier {
            let obj = &self.arg_name;
            let v = self.verify_field_tuple(&obj, &fields);
            Some(quote!({ if (#obj == undefined) return false; #(#v;)* return true; }))
        } else {
            None
        };

        QuoteMaker {
            body: quote!([#(#content),*]),
            verify,
            is_enum: false,
        }
    }
}
