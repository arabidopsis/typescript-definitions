#![allow(unused)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate typescript_definitions;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate wasm_bindgen;
extern crate proc_macro2;

use std::borrow::Cow;
use serde::de::value::Error;
use typescript_definitions::{TypescriptDefinition, TypeScriptify};
use proc_macro2::TokenStream;

use wasm_bindgen::prelude::*;

// NOTE:
// #[cfg(feature="test")] so that test functions 
// withing derive_typescript_definition are compiled...
// don't know why I can't just test for test!
// run these with `cargo test --features=test`

trait TypeScriptifyTrait {
    fn type_script_ify() -> &'static str;
}

// can't get access to typescript_definitions::patch!
// so we do our own.
fn patch(ts: proc_macro2::TokenStream) -> String {
    let s = ts.to_string(); // why do I have to do this?
    s.replace("[ ]","[  ]")
    .replace("{ }", "{  }")
}
// for type_script_ify
fn patcht(ts: proc_macro2::TokenStream) -> String {
   let s = ts.to_string();
    s.replace(" : ", ": ")
    .replace(" ;", ";")
    .replace(" < ", "<")
    .replace(" > ", ">")
    .replace("{ }", "{}")
    .replace("[ ]", "[]")
    .replace(" ;", ";")
    
}


#[cfg(feature="test")]
#[test]
fn unit_struct() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Unit;

    assert_eq!(Unit___typescript_definition(), patch(quote!{
        export type Unit = {};
    }));
}
#[cfg(feature="test")]
#[test]
fn newtype_struct() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Newtype(i64);

    assert_eq!(Newtype___typescript_definition(), patch(quote!{
        export type Newtype = number;
    }));
}
#[cfg(feature="test")]
#[test]
fn tuple_struct() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Tuple(i64, String);

    assert_eq!(Tuple___typescript_definition(), patch(quote!{
        export type Tuple = [number, string];
    }));
}
#[cfg(feature="test")]
#[test]
fn struct_with_borrowed_fields() {
    #[derive(Serialize, TypescriptDefinition, TypeScriptify)]
    struct Borrow<'a> {
        raw: &'a str,
        cow: Cow<'a, str>
    }

    assert_eq!(Borrow___typescript_definition(), patch(quote!{
       export type Borrow = {raw: string, cow: string };
    }));

}
#[cfg(feature="test")]
#[test]
fn struct_point_with_field_rename() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Point {
        #[serde(rename = "X")]
        x: i64,
        #[serde(rename = "Y")]
        y: i64,
    }

    assert_eq!(Point___typescript_definition(), patch(quote!{
        export type Point = {X: number, Y: number};
    }));
}
#[cfg(feature="test")]
#[test]
fn struct_with_array() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Point {

        x: [i64; 5],
        y: i64,
        z: Option<f64>,
    }

    assert_eq!(Point___typescript_definition(), patch(quote!{
        export type Point = {x: number[], y: number, z:  number | null  };
    }));
}
#[cfg(feature="test")]
#[test]
fn struct_with_tuple() {
    use std::collections::{HashMap,HashSet};

    #[derive(Serialize, TypescriptDefinition)]
    struct Point2 {

        x: (i64, String, [u128; 5]),
        y: i64,
        v: Vec<i32>,
        z: HashMap<String,i32>
    }

    assert_eq!(Point2___typescript_definition(), patch(quote!{
        export type Point2 = {x: [number, string, number[]], y: number, v: number[], z: Map<string,number>};
    }));
}
#[cfg(feature="test")]
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
    
    assert_eq!(Enum___typescript_definition(), patch(quote!{
        export type Enum = {kind: "Var1", fields: boolean}
        | {kind: "Var2", fields: number}
        | {kind: "Var3", fields: string};
    }));
}
#[cfg(feature="test")]
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

    assert_eq!(Enum___typescript_definition(), patch(quote!{
        export type  Enum = {kind: "V1"}
        | {kind: "V2"}
        | {kind: "V3"};
    }));
}
#[cfg(feature="test")]
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

    assert_eq!(Enum___typescript_definition(), patch(quote!{
        export type Enum = {kind: "V1", fields: [number, string]}
        | {kind: "V2", fields: [number, boolean]}
        | {kind: "V3", fields: [number, number]};
    }));
}
#[cfg(feature="test")]
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
    
    assert_eq!(Enum___typescript_definition(), patch(quote!{
        export type Enum = {kind: "V1",  Foo: boolean  }
        | {kind: "V2",  Bar: number, Baz: number  }
        | {kind: "V3",  Quux: string  };
    }));
}
#[cfg(feature="test")]
#[test]
fn enum_with_struct_and_tags() {
    #[derive(Serialize, TypescriptDefinition)]
    #[serde(tag="id", content="content")]
    enum Enum {
        V1 {
            foo: bool,
        },
        V2 {
            bar: i64,
            baz: u64,
        },
        V3 {
            quux: String,
        },
    }
    
    assert_eq!(Enum___typescript_definition(), patch(quote!{
         export type Enum = {id: "V1",  content: { foo: boolean  }}
        | {id: "V2", content: { bar: number, baz: number  }}
        | {id: "V3",  content: { quux: string  }};
    }));
}
#[cfg(feature="test")]
#[test]
fn struct_with_attr_refering_to_other_type() {
    #[derive(Serialize)]
    struct B<T> {q: T}

    #[derive(Serialize, TypescriptDefinition)]
    struct A {
        x : f64, /* simple */
        b: B<f64>,
        #[serde(rename="xxx")]
        c: Result<i32,&'static str>,
        d: Result<Option<i32>,String>,
    }
    assert_eq!(A___typescript_definition(), patch(quote!{
       export type A = { x: number ,b: B<number>, xxx: {Ok: number } | {Err: string}, d: {Ok: number | null} | { Err: string} };
    }));
}

#[test]
fn struct_typescriptify() {

    #[derive(TypeScriptify)]
    struct A {
        x : f64, /* simple */
        c: Result<i32,&'static str>,
        d: Result<Option<i32>,String>,
    }
    assert_eq!(A::type_script_ify(), patcht(quote!{
        export type A = { x: number ,c: {Ok: number } | {Err: string}, d: {Ok: number | null} | { Err: string} };
    }));
}

#[test]
fn cow_as_pig() {
    use std::borrow::Cow as Pig;

    #[derive(TypeScriptify)]
    struct S<'a> {
        pig: Pig<'a, str>,
        cow : ::std::borrow::Cow<'a, str>,
    }
    assert_eq!(S::type_script_ify(), patcht(quote!{
        export type S = { pig : Pig<string>, cow : string };
    }));

}
