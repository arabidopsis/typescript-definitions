#![allow(unused)]
#[macro_use]
extern crate typescript_definitions;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate wasm_bindgen;
// extern crate proc_macro2;
// extern crate serde;

mod patch;



use proc_macro2::TokenStream;
use serde::de::value::Error;
use std::borrow::Cow;
use typescript_definitions::{TypeScriptify, TypeScriptifyTrait, TypescriptDefinition};

use wasm_bindgen::prelude::*;
use patch::*;

// #[test]
fn type_scriptify_fields() {
    #[derive(Serialize, TypeScriptify)]
    struct S {
        a : i32,
        b : f64,
        c: String,
        // #[serde(rename="X")]
        d: Vec<String>
    }

    // assert_eq!(S::type_script_fields().unwrap(), vec!["a", "b", "c", "X"])
}
// #[test]
fn type_scriptify_generic_fields() {
    #[derive(Serialize, TypeScriptify)]
    struct S<'a,T> {
        a : i32,
        b : f64,
        c: String,
        #[serde(rename="X")]
        d: Vec<String>,
        e: &'a T,
    }

    // assert_eq!(S::<i32>::type_script_fields().unwrap(), vec!["a", "b", "c", "X", "e"])
}

// #[test]
fn type_scriptify_flatten() {
    #[derive(Serialize, TypeScriptify)]
    struct DDD {
        e : i32,
        f : f64
    }
    #[derive(Serialize, TypeScriptify)]
    struct SSS {
        a : i32,
        b : f64,
        // #[serde(flatten)]
        c : DDD
    }

    // assert_eq!(SSS::type_script_fields().unwrap(), vec!["a", "b", "c"])
}
