// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use serde_derive_internals::{ast, Ctxt};

use super::{derive_field, filter_visible, type_to_ts, QuoteMaker};

pub(crate) fn derive_struct<'a>(
    style: ast::Style,
    fields: &[ast::Field<'a>],
    container: &ast::Container,
    _cx: &Ctxt, // for error reporting
) -> (bool, QuoteMaker) {
    (
        false,
        match style {
            ast::Style::Struct => derive_struct_named_fields(fields, container),
            ast::Style::Newtype => derive_struct_newtype(&fields[0], container),
            ast::Style::Tuple => derive_struct_tuple(fields, container),
            ast::Style::Unit => derive_struct_unit(),
        },
    )
}

fn derive_struct_newtype<'a>(
    field: &ast::Field<'a>,
    _ast_container: &ast::Container,
) -> QuoteMaker {
    if field.attrs.skip_serializing() {
        return derive_struct_unit();
    }
    type_to_ts(&field.ty).into()
}

fn derive_struct_unit() -> QuoteMaker {
    quote!({}).into()
}

fn derive_struct_named_fields<'a>(
    fields: &[ast::Field<'a>],
    _ast_container: &ast::Container,
) -> QuoteMaker {
    let fields = filter_visible(fields);
    if fields.len() == 0 {
        return derive_struct_unit();
    }
    let content = fields.iter().map(|f| derive_field(f));
    quote!({#(#content),*}).into()
}

fn derive_struct_tuple<'a>(
    fields: &[ast::Field<'a>],
    _ast_container: &ast::Container,
) -> QuoteMaker {
    let fields = filter_visible(fields);
    if fields.len() == 0 {
        return derive_struct_unit();
    }
    let content = fields.iter().map(|f| type_to_ts(f.ty));
    quote!([#(#content),*]).into()
}
