#![allow(unused)]
// #[macro_use]
// extern crate typescript_definitions;
// #[macro_use]
// extern crate serde_derive;
// #[macro_use]
// extern crate quote;
// #[macro_use]
// extern crate wasm_bindgen;
// extern crate proc_macro2;
use typescript_definitions::{TypeScriptify, TypeScriptifyTrait, TypescriptDefinition};
// see https://github.com/graphql-rust/graphql-client/issues/176
use serde_derive::*;

use serde::Serialize;
use quote::quote;
use proc_macro2::TokenStream;
// use serde::de::value::Error;
use std::borrow::Cow;

mod patch;
use patch::{patch, patcht};
use wasm_bindgen::prelude::*;

// #[test]
fn type_scriptify_fields() {
    #[derive(Serialize, TypeScriptify)]
    struct S {
        a: i32,
        b: f64,
        c: String,
        // #[serde(rename="X")]
        d: Vec<String>,
    }

    // assert_eq!(S::type_script_fields().unwrap(), vec!["a", "b", "c", "X"])
}
// #[test]
fn type_scriptify_generic_fields() {
    #[derive(Serialize, TypeScriptify)]
    struct S<'a, T> {
        a: i32,
        b: f64,
        c: String,
        #[serde(rename = "X")]
        d: Vec<String>,
        e: &'a T,
    }

    // assert_eq!(S::<i32>::type_script_fields().unwrap(), vec!["a", "b", "c", "X", "e"])
}

// #[test]
fn type_scriptify_flatten() {
    #[derive(Serialize, TypeScriptify)]
    struct DDD {
        e: i32,
        f: f64,
    }
    #[derive(Serialize, TypeScriptify)]
    struct SSS {
        a: i32,
        b: f64,
        // #[serde(flatten)]
        c: DDD,
    }

    // assert_eq!(SSS::type_script_fields().unwrap(), vec!["a", "b", "c"])
}
#[test]
fn as_byte_string() {
    use serde_json;
    // use serde_json::Error;
    #[derive(Serialize, TypeScriptify)]
    struct S {
         #[serde(serialize_with="typescript_definitions::as_byte_string")]
        image : Vec<u8>
    }

    let s = S { image: vec![1,2,3,4,5]};
    assert_eq!(serde_json::to_string(&s).unwrap(), "{\"image\":\"\\\\x01\\\\x02\\\\x03\\\\x04\\\\x05\"}");

}

#[test]
fn untagged_enum() {
    use serde_json;
    // use serde_json::Error;
    #[derive(Serialize, TypeScriptify)]
    #[serde(untagged)]
    enum Untagged {
        V1 { id: i32, attr : String} ,
        V2 { id: i32, attr2: Vec<String> }
    }

    assert_eq!(Untagged::type_script_ify().replace("\n  ",""), patcht(quote!{
        export type Untagged = {id: number, attr: string} | {id: number, attr2 : string[] };

    }));

}

#[test]
fn external_enum() {
    use serde_json;
    // use serde_json::Error;
    #[derive(Serialize, TypeScriptify)]
    enum External {
        V1 { id: i32, attr : String} ,
        V2 { id: i32, attr2: Vec<String> }
    }

    assert_eq!(External::type_script_ify().replace("\n  ",""), patcht(quote!{
        export type External = { V1: {id: number, attr: string} } | { V2: {id: number, attr2 : string[] }};

    }));

}