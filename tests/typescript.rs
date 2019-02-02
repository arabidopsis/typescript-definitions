#![allow(unused)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate wasm_typescript_definition2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate wasm_bindgen;

use std::borrow::Cow;
use serde::de::value::Error;
use wasm_typescript_definition2::TypescriptDefinition;
use wasm_bindgen::prelude::*;

#[test]
fn unit_struct() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Unit;

    assert_eq!(Unit___typescript_definition(), quote!{
        {}
    }.to_string());
}

#[test]
fn newtype_struct() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Newtype(i64);

    assert_eq!(Newtype___typescript_definition(), quote!{
        number
    }.to_string());
}

#[test]
fn tuple_struct() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Tuple(i64, String);

    assert_eq!(Tuple___typescript_definition(), quote!{
        [number, string]
    }.to_string());
}

#[test]
fn struct_with_borrowed_fields() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Borrow<'a> {
        raw: &'a str,
        cow: Cow<'a, str>
    }

    assert_eq!(Borrow___typescript_definition(), quote!{
        {"raw": string, "cow": string }
    }.to_string());
}

#[test]
fn struct_point_with_field_rename() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Point {
        #[serde(rename = "X")]
        x: i64,
        #[serde(rename = "Y")]
        y: i64,
    }

    assert_eq!(Point___typescript_definition(), quote!{
        {"X": number, "Y": number}
    }.to_string());
}
#[test]
fn struct_with_array() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Point {

        x: [i64; 5],
        y: i64,
    }

    assert_eq!(Point___typescript_definition(), quote!{
        {"x": number[], "y": number}
    }.to_string());
}
#[test]
fn struct_with_tuple() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Point2 {

        x: (i64, String, [u128; 5]),
        y: i64,
        v: Vec<i32>,
    }

    assert_eq!(Point2___typescript_definition(), quote!{
        {"x": [number, string, number[]], "y": number, "v": number[]}
    }.to_string());
}
#[test]
fn enum_with_renamed_newtype_variants() {
    #[derive(Serialize, TypescriptDefinition)]
    enum Enum {
        #[serde(rename = "Var1")]
        #[allow(unused)]
        V1(bool),
        #[serde(rename = "Var2")]
        #[allow(unused)]
        V2(i64),
        #[serde(rename = "Var3")]
        #[allow(unused)]
        V3(String),
    }
    
    assert_eq!(Enum___typescript_definition(), quote!{
        | {"tag": "Var1", "fields": boolean}
        | {"tag": "Var2", "fields": number}
        | {"tag": "Var3", "fields": string}
    }.to_string());
}

#[test]
fn enum_with_unit_variants() {
    #[derive(Serialize, TypescriptDefinition)]
    enum Enum {
        #[allow(unused)]
        V1,
        #[allow(unused)]
        V2,
        #[allow(unused)]
        V3,
    }

    assert_eq!(Enum___typescript_definition(), quote!{
        | {"tag": "V1"}
        | {"tag": "V2"}
        | {"tag": "V3"}
    }.to_string());
}

#[test]
fn enum_with_tuple_variants() {
    #[derive(Serialize, TypescriptDefinition)]
    enum Enum {
        #[allow(unused)]
        V1(i64, String),
        #[allow(unused)]
        V2(i64, bool),
        #[allow(unused)]
        V3(i64, u64),
    }

    assert_eq!(Enum___typescript_definition(), quote!{
        | {"tag": "V1", "fields": [number, string]}
        | {"tag": "V2", "fields": [number, boolean]}
        | {"tag": "V3", "fields": [number, number]}
    }.to_string());
}

#[test]
fn enum_with_struct_variants_and_renamed_fields() {
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
    
    assert_eq!(Enum___typescript_definition(), quote!{
        | {"tag": "V1", "fields": { "Foo": boolean } }
        | {"tag": "V2", "fields": { "Bar": number, "Baz": number } }
        | {"tag": "V3", "fields": { "Quux": string } }
    }.to_string());
}
