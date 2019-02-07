extern crate proc_macro2;
extern crate quote;

use proc_macro2::TokenStream;
use quote::ToTokens;
// use std::str::ToString;

enum QuoteT<'a> {
    Tokens(TokenStream),
    Builder(Box<Fn() -> TokenStream + 'a>), //Builder(fn() -> TokenStream)
                                            //Builder(Fn() -> TokenStream)
}

impl ToTokens for QuoteT<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            QuoteT::Tokens(t) => t.to_tokens(tokens),
            QuoteT::Builder(f) => f().to_tokens(tokens),
        }
    }
}
impl ToString for QuoteT<'_> {
    fn to_string(&self) -> String {
        match self {
            QuoteT::Tokens(t) => t.to_string(),
            QuoteT::Builder(f) => f().to_string(),
        }
    }
}
impl QuoteT<'_> {
    fn from_quote<'a>(s: TokenStream) -> QuoteT<'a> {
        QuoteT::Tokens(s)
    }
    fn from_closure<'a, F: Fn() -> TokenStream + 'a>(f: F) -> QuoteT<'a> {
        QuoteT::Builder(Box::new(f))
    }
}

fn make_closure<'a>() -> QuoteT<'a> {
    let s = vec![
        QuoteT::from_quote(quote!(a b)),
        QuoteT::from_quote(quote!(c b)),
    ];
    let f = move || {
        let t = &s; // this needs to be a ref....!
        quote! (some more #(#t),*)
    };
    QuoteT::from_closure(f)
}
