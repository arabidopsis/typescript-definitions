// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use serde_derive_internals::{ast};

use super::{derive_field, filter_visible, type_to_ts, QuoteMaker, quotet::Tbuild, QuoteT, ParseContext};

pub(crate) fn derive_struct<'a>(
    style: ast::Style,
    fields: &[ast::Field<'a>],
    container: &ast::Container,
    ctxt: &ParseContext, // for error reporting
) -> QuoteMaker {
        match style {
            ast::Style::Struct => derive_struct_named_fields(fields, container, ctxt),
            ast::Style::Newtype => derive_struct_newtype(&fields[0], container, ctxt),
            ast::Style::Tuple => derive_struct_tuple(fields, container, ctxt),
            ast::Style::Unit => derive_struct_unit(),
        }
}

fn derive_struct_newtype<'a>(
    field: &ast::Field<'a>,
    _ast_container: &ast::Container,
    ctxt : &ParseContext,
) -> QuoteMaker {
    if field.attrs.skip_serializing() {
        return derive_struct_unit();
    }
    type_to_ts(&field.ty, ctxt).into()
}

fn derive_struct_unit() -> QuoteMaker {
    quote!({}).into()
}

struct Fields {
    fields : Vec<String>,
    body : Vec<QuoteT>
}
impl Tbuild for Fields {
    fn build(&self) -> QuoteT {
        let content = &self.body;
        quote!({#(#content),*})
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
fn derive_struct_named_fields<'a>(
    fields: &[ast::Field<'a>],
    _ast_container: &ast::Container,
    ctxt : &ParseContext,
) -> QuoteMaker {
    let fields = filter_visible(fields);
    if fields.len() == 0 {
        return derive_struct_unit();
    };
    let names = fields.iter().map(|f| f.attrs.name().serialize_name()).collect::<Vec<_>>();
    let content = fields.iter().map(|f| derive_field(f, ctxt)).collect::<Vec<_>>();
    QuoteMaker::from_builder(Fields { fields: names, body: content })

}

fn derive_struct_tuple<'a>(
    fields: &[ast::Field<'a>],
    _ast_container: &ast::Container,
    ctxt : &ParseContext,
) -> QuoteMaker {
    let fields = filter_visible(fields);
    if fields.len() == 0 {
        return derive_struct_unit();
    }
    let content = fields.iter().map(|f| type_to_ts(f.ty, ctxt));
    quote!([#(#content),*]).into()
}
