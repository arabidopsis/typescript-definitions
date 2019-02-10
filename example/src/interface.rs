#![allow(unused)]
extern crate serde_derive;
extern crate typescript_definitions;
extern crate wasm_bindgen;

use serde_derive::{Serialize};

use typescript_definitions::{TypeScriptify, TypescriptDefinition};

use wasm_bindgen::prelude::*;

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

#[derive(Serialize, TypescriptDefinition, TypeScriptify)]
pub enum Enum {
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
// #[derive(Serialize)]
#[derive(Serialize, TypescriptDefinition, TypeScriptify)]
pub struct Value<T> {
    value: T,
}

#[derive(TypescriptDefinition, Serialize, TypeScriptify)]
#[serde(tag = "tag", content = "fields")]
pub enum FrontendMessage {
    Init {
        id: String,
    },
    ButtonState {
        selected: Vec<String>,
        time: u32,
        other: Option<String>,
    },
    Render {
        html: String,
        time: u32,
        other: Result<&'static str, i32>,
    },
    Stuff {
        borrow: Value<i32>,
    },
}

use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Serialize, TypescriptDefinition, TypeScriptify)]
pub struct Borrow<'a> {
    raw: &'a str,
    cow: Cow<'a, str>,
    map: HashMap<String, i32>,
    
}

#[derive(Serialize, TypescriptDefinition, TypeScriptify)]
pub struct MyBytes<'a> {
    #[serde(with="serde_bytes")]
    pub buffer: &'a [u8]
    
}
