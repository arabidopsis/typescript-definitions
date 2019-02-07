// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use serde_derive_internals::{ast, Ctxt};

use super::{derive_field, type_to_ts, QuoteT};

pub(crate) fn derive_struct<'a>(
    style: ast::Style,
    fields: &[ast::Field<'a>],
    container: &ast::Container,
    _cx: &Ctxt, // for error reporting
) -> (bool, QuoteT) {
    (
        false,
        match style {
            ast::Style::Struct => derive_struct_named_fields(fields, container),
            ast::Style::Newtype => derive_struct_newtype(fields, container),
            ast::Style::Tuple => derive_struct_tuple(fields, container),
            ast::Style::Unit => derive_struct_unit(container),
        },
    )
}

fn derive_struct_newtype<'a>(
    fields: &[ast::Field<'a>],
    _attr_container: &ast::Container,
) -> QuoteT {
    type_to_ts(&fields[0].ty)
}

fn derive_struct_unit(_attr_container: &ast::Container) -> QuoteT {
    quote! {
        {}
    }
}

fn derive_struct_named_fields<'a>(
    fields: &[ast::Field<'a>],
    _attr_container: &ast::Container,
) -> QuoteT {
    let content = fields.iter().map(|field| derive_field(field));

    quote!({#(#content),*})
}

fn derive_struct_tuple<'a>(fields: &[ast::Field<'a>], _attr_container: &ast::Container) -> QuoteT {
    let content = fields.iter().map(|field| type_to_ts(field.ty));

    quote!([#(#content),*])
}
