use failure::Fail;
use pest::Parser;

use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Fail, Debug)]
#[fail(display = "{}", _0)]
pub struct SelectorParseError(pest::error::Error<Rule>);

impl SelectorParseError {
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

pub struct Typescript;

impl Typescript {
    pub fn parse(&self, typescript: &str) -> Result<(), SelectorParseError> {
        let pair = TypescriptParser::parse(Rule::typescript, typescript)
            .map_err(SelectorParseError)?
            .next() // skip SOI
            .unwrap();

        for item in pair.into_inner() {
            match item.as_rule() {
                Rule::EOI => break,
                other => assert_eq!(other, Rule::item),
            }
            self.parse_item(item)?
        }

        Ok(())
    }
    fn parse_item<'a>(&self, item: Pair<'a, Rule>) -> Result<(), SelectorParseError> {
        let mut i = item.into_inner();
        let (singleton, array) = (i.next().unwrap(), i.next().unwrap());

        for singleton_pair in singleton.into_inner() {
            eprintln!("HERE2 {:?}", singleton_pair.as_rule());
            match singleton_pair.as_rule() {
                Rule::map => self.parse_map(singleton_pair)?,
                Rule::str => self.parse_str(singleton_pair)?,
                Rule::union => self.parse_union(singleton_pair)?,
                Rule::tuple => self.parse_union(singleton_pair)?,
                Rule::typ => self.parse_typ(singleton_pair)?,
                _ => unreachable!(),
            }
        }
        Ok(())
    }
    fn parse_typ<'a>(&self, typ: Pair<'a, Rule>) -> Result<(), SelectorParseError> {
        eprintln!("type {}", typ.as_str());
        Ok(())
    }
    fn parse_map<'a>(&self, map: Pair<'a, Rule>) -> Result<(), SelectorParseError> {
        let mut i = map.into_inner();
        let (typ, item) = (i.next().unwrap(), i.next().unwrap());
        self.parse_typ(typ)?;
        self.parse_item(item)?;
        Ok(())
    }
    fn parse_union<'a>(&self, union: Pair<'a, Rule>) -> Result<(), SelectorParseError> {
        for item in union.into_inner() {
            match item.as_rule() {
                Rule::item => self.parse_item(item)?,
                _ => unreachable!(),
            }
        }
        Ok(())
    }
    fn parse_tuple<'a>(&self, tuple: Pair<'a, Rule>) -> Result<(), SelectorParseError> {
        for item in tuple.into_inner() {
            match item.as_rule() {
                Rule::item => self.parse_item(item)?,
                _ => unreachable!(),
            }
        }
        Ok(())
    }
    fn parse_str<'a>(&self, pair: Pair<'a, Rule>) -> Result<(), SelectorParseError> {
        for item in pair.into_inner() {
            match item.as_rule() {
                Rule::ident => {}
                Rule::item => self.parse_item(item)?,
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod parser {
    use super::Typescript;
    #[test]
    fn typescript_parser() {
        let t = Typescript {};
        match t.parse(&"[number, string]|{ [key: number]: string}[][] | {a: number} | (number|{a:{b:number}})") {
            Ok(_) => {},
            Err(msg) => assert!(false, msg)
        }
    }

}
