#![allow(unused)]

use typescript_definitions::{TypeScriptify, TypeScriptifyTrait, TypescriptDefinition};

use serde::Serialize;
// use serde::de::value::Error;
use insta::assert_snapshot_matches;
use wasm_bindgen::prelude::*;

#[test]
fn as_byte_string() {
    use serde_json;
    // use serde_json::Error;
    #[derive(Serialize, TypeScriptify)]
    struct S {
        #[serde(serialize_with = "typescript_definitions::as_byte_string")]
        image: Vec<u8>,
    }

    let s = S {
        image: vec![1, 2, 3, 4, 5, 244],
    };
    assert_snapshot_matches!(
        serde_json::to_string(&s).unwrap(),
        @r###"{"image":"\\x01\\x02\\x03\\x04\\x05\\xf4"}"###

    )
}

#[test]
fn untagged_enum() {
    use serde_json;
    // use serde_json::Error;
    #[derive(Serialize, TypeScriptify)]
    #[serde(untagged)]
    enum Untagged {
        V1 { id: i32, attr: String },
        V2 { id: i32, attr2: Vec<String> },
    }

    assert_snapshot_matches!(
        Untagged::type_script_ify(),
        @r###"export type Untagged = 
 | { id: number; attr: string } 
 | { id: number; attr2: string[] };"###

    )
}

#[test]
fn external_enum() {
    use serde_json;
    // use serde_json::Error;
    #[derive(Serialize, TypeScriptify)]
    /// Has documentation.
    enum External {
        V1 { id: i32, attr: String },
        V2 { id: i32, attr2: Vec<String> },
    }

    assert_snapshot_matches!(
    External::type_script_ify(),
        @r###"// Has documentation.
export type External = 
 | { V1: { id: number; attr: string } } 
 | { V2: { id: number; attr2: string[] } };"###
    )
}
