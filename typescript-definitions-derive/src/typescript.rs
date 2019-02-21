#![allow(unused)]
use super::ident_from_str;
use super::patch::{eq, nl};
use failure::Fail;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::quote;
use std::cell::RefCell;

#[derive(Fail, Debug)]
#[fail(display = "{}", _0)]
pub struct TypescriptParseError(pest::error::Error<Rule>);

impl TypescriptParseError {
    /// Return the column of where the error ocurred.
    #[allow(unused)]
    pub fn column(&self) -> usize {
        match self.0.line_col {
            pest::error::LineColLocation::Pos((_, col)) => col,
            pest::error::LineColLocation::Span((_, col), _) => col,
        }
    }
}

#[derive(Parser)]
#[grammar = "typescript.pest"]
struct TypescriptParser;

pub struct Typescript {
    only_first: bool,
    var: RefCell<i32>,
}

impl Typescript {
    pub fn new() -> Self {
        Typescript {
            only_first: false,
            var: RefCell::new(0),
        }
    }
    pub fn parse(
        &self,
        obj: &TokenStream,
        typescript: &str,
    ) -> Result<TokenStream, TypescriptParseError> {
        let pair = TypescriptParser::parse(Rule::typescript, typescript)
            .map_err(TypescriptParseError)?
            .next() // skip SOI
            .unwrap();
        let mut content = vec![];
        for item in pair.into_inner() {
            match item.as_rule() {
                Rule::EOI => break,
                other => assert_eq!(other, Rule::item),
            }
            content.push(self.parse_item(obj, item)?);
        }
        let newl = nl();
        // obj can't be null or undefined
        if content.len() == 1 {
            Ok(quote!(
                {

                    #(#content)*
                    #newl return true;
                }
            ))
        } else {
            let nl = (0..content.len()).map(|_| quote!(#newl));
            Ok(quote!(
                {

                    #( #nl if ( ( () => { #content } )() ) return true; )*
                    #newl return false;
                }
            ))
        }
    }
    fn parse_item<'a>(
        &self,
        obj: &TokenStream,
        item: Pair<'a, Rule>,
    ) -> Result<TokenStream, TypescriptParseError> {
        let mut i = item.into_inner();
        let (singleton, array) = (i.next().unwrap(), i.next().unwrap());
        let mut content = vec![];
        let narr = array.as_str().len() / 2;
        let val = obj; // self.pushvar();
        let mut is_union = false;

        for singleton_pair in singleton.into_inner() {
            content.push(match singleton_pair.as_rule() {
                Rule::map => self.parse_map(&val, singleton_pair)?,
                Rule::str => self.parse_struct(&val, singleton_pair)?,
                Rule::union => {
                    is_union = true;
                    self.parse_union(&val, singleton_pair)?
                }
                Rule::tuple => self.parse_tuple(&val, singleton_pair)?,
                Rule::typ => self.parse_typ(&val, singleton_pair)?,
                _ => unreachable!(),
            });
        }
        assert!(content.len() == 1);
        if narr == 0 {
            // self.popvar();
            Ok(quote!(
                {
                    if (#obj == undefined) return false;
                    #(#content)*
                }
            ))
        } else {
            let brk = if self.only_first {
                quote!(break;)
            } else {
                quote!()
            };
            let test = if is_union {
                quote!(#(if ( !( () => { #content } )() ) return false; )*)
            } else {
                quote!( #(#content)* )
            };
            let mut vinner = self.pushvar();
            let mut inner = quote!(
                {
                    if (!Array.isArray(#vinner)) return false;
                    for (let #val of #vinner) {
                        #test
                        #brk;
                    }
                }
            );
            for i in 0..narr - 1 {
                let vnext = self.pushvar();
                inner = quote!(
                if (!Array.isArray(#vnext)) return false;
                for (let #vinner of #vnext) {
                    #inner
                });
                vinner = vnext;
            }
            for i in 0..narr {
                self.popvar()
            }
            Ok(quote!(let #vinner = #obj; #inner; return true;))
        }
    }
    fn parse_typ<'a>(
        &self,
        obj: &TokenStream,
        typ: Pair<'a, Rule>,
    ) -> Result<TokenStream, TypescriptParseError> {
        let typ = typ.as_str();
        let eq = eq();
        Ok(quote!(
            if (!(typeof #obj #eq #typ)) return false;
        ))
    }
    fn parse_map<'a>(
        &self,
        obj: &TokenStream,
        map: Pair<'a, Rule>,
    ) -> Result<TokenStream, TypescriptParseError> {
        let mut i = map.into_inner();
        let (typ, item) = (i.next().unwrap(), i.next().unwrap());
        let k = typ.as_str();
        // let typ = self.parse_typ(typ)?;
        let val = self.pushvar();
        let v = self.parse_item(&val, item)?;
        let eq = eq();
        let kval = self.pushvar();
        let k = if k == "number" {
            quote! {
                if (+#kval #eq NaN) return false;
            }
        } else {
            //self.verify_type(&quote!(k), &ts.args[0]);
            // always going to be a string
            quote!()
        };
        let brk = if self.only_first {
            quote!(break;)
        } else {
            quote!()
        };
        self.popvar();
        self.popvar();
        // obj is definitely not undefined... but it might be null...
        Ok(quote!(
            if (#obj #eq null || !(typeof #obj #eq "object")) return false;
            for (let #kval in #obj) {
                let #val = #obj[#kval];
                #k;
                #v;
                #brk
            }
        ))
    }
    fn parse_union<'a>(
        &self,
        obj: &TokenStream,
        union: Pair<'a, Rule>,
    ) -> Result<TokenStream, TypescriptParseError> {
        let mut content = vec![];
        let val = self.pushvar();
        for item in union.into_inner() {
            match item.as_rule() {
                Rule::item => content.push(self.parse_item(&val, item)?),
                _ => unreachable!(),
            }
        }
        let newl = nl();
        let nl = (0..content.len()).map(|_| quote!(#newl));
        self.popvar();
        // obj can't be null or undefined
        Ok(quote!(
            {
                if (#obj == undefined) return false;

                #( #nl if ( ( () => { #content } )() ) return true; )*
                #newl return false;
            }
        ))
    }
    fn parse_tuple<'a>(
        &self,
        obj: &TokenStream,
        tuple: Pair<'a, Rule>,
    ) -> Result<TokenStream, TypescriptParseError> {
        let mut content = vec![];
        let eq = eq();
        let val = self.pushvar();
        for (i, item) in tuple.into_inner().enumerate() {
            let i = Literal::usize_unsuffixed(i);
            let n = quote!(#obj[#i]);

            match item.as_rule() {
                Rule::item => {
                    let verify = self.parse_item(&val, item)?;

                    content.push(quote! {
                        if (#n #eq undefined) return false;
                        {
                            const #val = #n;
                            #verify;
                        }
                    });
                }
                _ => unreachable!(),
            }
        }
        self.popvar();
        let len = Literal::usize_unsuffixed(content.len());
        Ok(quote!(if (!Array.isArray(#obj) || !(#obj.length #eq #len)) return false; #(#content)* ))
    }
    fn parse_struct<'a>(
        &self,
        obj: &TokenStream,
        pair: Pair<'a, Rule>,
    ) -> Result<TokenStream, TypescriptParseError> {
        let mut keys = vec![];
        let mut values = vec![];
        let val = self.pushvar();
        for item in pair.into_inner() {
            match item.as_rule() {
                Rule::ident => keys.push(ident_from_str(&item.as_str())),
                Rule::item => values.push(self.parse_item(&val, item)?),
                _ => unreachable!(),
            }
        }
        let mut ret = vec![];
        let eq = eq();
        for (n, v) in keys.iter().zip(values) {
            ret.push(quote! {
                if (#obj.#n #eq undefined) return false;
                {
                    const #val = #obj.#n;
                    #v;
                }
            });
        }
        self.popvar();
        Ok(quote!(#(#ret)*;))
    }

    fn pushvar(&self) -> TokenStream {
        let mut var = self.var.borrow_mut();
        *var += 1;

        let n = ident_from_str(&format!("val{}", *var));
        quote!(#n)
    }
    fn popvar(&self) {
        let mut var = self.var.borrow_mut();
        *var -= 1;
    }
}

#[cfg(test)]
mod parser {
    use super::Typescript;
    use crate::patch::patch;
    use quote::quote;
    //#[test]
    fn typescript_parser() {
        let t = Typescript::new();
        match t.parse(&quote!(obj), &"[number, string]|{ [key: number]: string}[][] | {a: number} | (number|{a:{b:number}})") {
            Ok(q) => {eprintln!("{}", patch(&q.to_string()))},
            Err(msg) => assert!(false, msg)
        }
    }
    #[test]
    fn typescript_parser2() {
        let t = Typescript::new();
        match t.parse(&quote!(obj), &"[number, string][]") {
            Ok(q) => eprintln!("{}", patch(&q.to_string())),
            Err(msg) => assert!(false, msg),
        }
    }
}
