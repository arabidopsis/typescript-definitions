// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use quote::quote;
use serde_derive_internals::ast;

use super::{filter_visible, verify::Verify, Attrs, ParseContext, QuoteMaker};

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

        let verify = if true || self.global_attrs.verify {
            let attrs = Attrs::from_field(field, self.ctxt);
            let verify = Verify {
                attrs,
                ctxt: self,
                field: field,
            };
            let v = verify.verify_type(&self.verify, &field.ty);
            Some(quote!({ #v; return true; }))
        } else {
            None
        };

        QuoteMaker {
            body: self.field_to_ts(field),
            verify: verify,
            is_enum: false,
        }
    }

    fn derive_struct_unit(&self) -> QuoteMaker {
        let obj = &self.verify;
        QuoteMaker {
            body: quote!({}),
            verify: Some(quote!({ if (#obj == null) return false; return true; })),
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

        let verify = self.verify_fields(&self.verify, &fields);
        let obj = &self.verify;
        QuoteMaker {
            body: quote!({#(#content);*}),
            verify: Some(quote!({ if (#obj == null) return false; #(#verify;)* return true; })),
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
        let verify = self.verify_field_tuple(&self.verify, &fields);
        let obj = &self.verify;
        QuoteMaker {
            body: quote!([#(#content),*]),
            verify: Some(quote!({ if (#obj == null) return false; #(#verify;)* return true; })),
            is_enum: false,
        }
    }
}
