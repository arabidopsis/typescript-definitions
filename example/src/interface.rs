#![allow(unused)]

use serde::Serialize;
use typescript_definitions::TypescriptDefinition;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Serialize, TypescriptDefinition)]
struct Foo {
    a: i32,
    b: i8,
}

#[derive(Serialize, TypescriptDefinition)]
struct Bar {
    baz: Foo,
    quux: Vec<i8>,
}
