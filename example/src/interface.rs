
// extern crate serde;
extern crate serde_derive;
extern crate typescript_definitions;
extern crate wasm_bindgen;


use::wasm_bindgen::prelude::*;
use::serde_derive::{Serialize};
use::typescript_definitions::TypescriptDefinition;
use::typescript_definitions::TypeScriptify;

pub trait TypeScriptifyTrait {
    fn type_script_ify() -> &'static str;
}

#[derive(Serialize, TypescriptDefinition)]
pub struct Newtype(i64);


#[derive(Serialize, TypescriptDefinition, TypeScriptify, Debug)]
pub struct Point {
    #[serde(rename = "X")]
    pub x: i64,
    #[serde(rename = "Y")]
    pub y: i64,
    pub z: i64,
}


#[derive(Serialize, TypescriptDefinition)]
enum Enum {
    #[allow(unused)]
    V1 {
        #[serde(rename = "Foo")]
        foo: bool,
    },
    #[allow(unused)]
    V2 {
        #[serde(rename = "Bar")]
        bar: i64,
        #[serde(rename = "Baz")]
        baz: u64,
    },
    #[allow(unused)]
    V3 {
        #[serde(rename = "Quux")]
        quux: String,
    },
}


#[derive(TypescriptDefinition, Serialize, TypeScriptify)]
#[serde(tag = "tag", content = "fields")]
pub enum FrontendMessage {
  Init { id: String, },
  ButtonState { selected: Vec<String>, time: u32, },
  Render { html: String, time: u32, },
}

use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Serialize, TypescriptDefinition, TypeScriptify)]
pub struct Borrow<'a> {
    raw: &'a str,
    cow: Cow<'a, str>,
    map : HashMap<String, i32>
}
