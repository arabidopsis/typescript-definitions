use quote::quote;
use syn::{self, Attribute, Lit, Meta, MetaNameValue};

#[derive(Debug)]
pub struct Comment {
    pub text: String,
}

#[derive(Debug)]
pub struct Attrs {
    pub comments: Vec<Comment>,
}

impl Attrs {
    pub fn new() -> Attrs {
        Attrs { comments: vec![] }
    }
    pub fn push_doc_comment(&mut self, attrs: &[Attribute]) {
        let doc_comments = attrs
            .iter()
            .filter_map(|attr| {
                let path = &attr.path;
                match quote!(#path).to_string() == "doc" {
                    true => attr.interpret_meta(),
                    false => None,
                }
            })
            .filter_map(|attr| {
                use Lit::*;
                use Meta::*;
                if let NameValue(MetaNameValue {
                    ident, lit: Str(s), ..
                }) = attr
                {
                    if ident != "doc" {
                        return None;
                    }
                    let value = s.value();
                    let text = value
                        .trim_start_matches("//!")
                        .trim_start_matches("///")
                        .trim_start_matches("/*!")
                        .trim_start_matches("/**")
                        .trim_end_matches("*/")
                        .trim();
                    if text.is_empty() {
                        None
                    } else {
                        Some(text.to_string())
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if doc_comments.is_empty() {
            return;
        }

        let merged_lines = doc_comments
            .iter()
            .map(|s| format!("// {}", s.trim()))
            .collect::<Vec<_>>()
            .join("\n");

        self.comments.push(Comment { text: merged_lines });
    }
}
