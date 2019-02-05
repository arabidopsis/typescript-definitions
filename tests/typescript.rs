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

use std::borrow::Cow;
use serde::de::value::Error;
use typescript_definitions::{TypescriptDefinition, TypeScriptify};
use wasm_bindgen::prelude::*;

trait TypeScriptifyTrait {
    fn type_script_ify() -> &'static str;
}

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
    #[derive(Serialize, TypescriptDefinition, TypeScriptify)]
    struct Borrow<'a> {
        raw: &'a str,
        cow: Cow<'a, str>
    }

    assert_eq!(Borrow___typescript_definition(), quote!{
        {raw: string, cow: string }
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
        {X: number, Y: number}
    }.to_string());
}
#[test]
fn struct_with_array() {
    #[derive(Serialize, TypescriptDefinition)]
    struct Point {

        x: [i64; 5],
        y: i64,
        z: Option<f64>,
    }

    assert_eq!(Point___typescript_definition(), quote!{
        {x: number[], y: number, z:  number | null  }
    }.to_string());
}
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

    assert_eq!(Point2___typescript_definition(), quote!{
        {x: [number, string, number[]], y: number, v: number[], z: Map<string,number>}
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
         {kind: "Var1", fields: boolean}
        | {kind: "Var2", fields: number}
        | {kind: "Var3", fields: string}
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
         {kind: "V1"}
        | {kind: "V2"}
        | {kind: "V3"}
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
         {kind: "V1", fields: [number, string]}
        | {kind: "V2", fields: [number, boolean]}
        | {kind: "V3", fields: [number, number]}
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
         {kind: "V1",  Foo: boolean  }
        | {kind: "V2",  Bar: number, Baz: number  }
        | {kind: "V3",  Quux: string  }
    }.to_string());
}

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
    
    assert_eq!(Enum___typescript_definition(), quote!{
         {id: "V1",  content: { foo: boolean  }}
        | {id: "V2", content: { bar: number, baz: number  }}
        | {id: "V3",  content: { quux: string  }}
    }.to_string());
}

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
    assert_eq!(A___typescript_definition(), quote!{
        { x: number ,b: B<number>, xxx: number | string, d: number | null | string }
    }.to_string());
}

#[test]
fn struct_typescriptify() {

    #[derive(TypeScriptify)]
    struct A {
        x : f64, /* simple */
        c: Result<i32,&'static str>,
        d: Result<Option<i32>,String>,
    }
    assert_eq!(A::type_script_ify(), &quote!{
        export type A = { x: number ,c: number | string, d: number | null | string } ;
    }.to_string().replace(" : ", ": "));
}

#[test]
fn cow_as_pig() {
    use std::borrow::Cow as Pig;

    #[derive(TypeScriptify)]
    struct S<'a> {
        pig: Pig<'a, str>,
        cow : ::std::borrow::Cow<'a, str>,
    }
    assert_eq!(S::type_script_ify(), quote!{
        export type S = { pig : Pig<string>, cow : string } ;
    }.to_string().replace(" : ", ": "));

}