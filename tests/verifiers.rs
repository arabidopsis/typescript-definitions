#![allow(unused)]
use std::process;

use typescript_definitions::{TypeScriptify, TypeScriptifyTrait, TypescriptDefinition};

use serde::Serialize;
// use serde::de::value::Error;
use insta::assert_snapshot_matches;
use wasm_bindgen::prelude::*;

use std::io::Write;
use std::process::{Command, Stdio};
pub fn prettier(s: &str) -> String {
    let mut child = Command::new("prettier")
        .arg("--parser")
        .arg("typescript")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute prettier");

    // hopefully the pipe buffers don't fill up :)
    {
        // limited borrow of stdin
        let stdin = child.stdin.as_mut().expect("failed to get stdin");
        stdin
            .write_all(s.as_bytes())
            .expect("failed to write to stdin");
    }

    let output = child
        .wait_with_output()
        .expect("failed to wait on prettier");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn verify_untagged_enum() {
    use serde_json;
    // use serde_json::Error;
    #[derive(Serialize, TypeScriptify)]
    #[serde(untagged)]
    enum Untagged {
        V1 { id: i32, attr: String },
        V2 { id: i32, attr2: Vec<String> },
    }
    let verify_untagged_enum = prettier(&Untagged::type_script_verify().unwrap());
    assert_snapshot_matches!(verify_untagged_enum,
        @r###"export const isa_Untagged = (obj: any): obj is Untagged => {
  if (obj == undefined) return false;
  if (
    (() => {
      if (obj.id === undefined) return false;
      {
        const val = obj.id;
        if (!(typeof val === "number")) return false;
      }
      if (obj.attr === undefined) return false;
      {
        const val = obj.attr;
        if (!(typeof val === "string")) return false;
      }
      return true;
    })()
  )
    return true;
  if (
    (() => {
      if (obj.id === undefined) return false;
      {
        const val = obj.id;
        if (!(typeof val === "number")) return false;
      }
      if (obj.attr2 === undefined) return false;
      {
        const val = obj.attr2;
        if (!Array.isArray(val)) return false;
        for (let x of val) {
          if (!(typeof x === "string")) return false;
        }
      }
      return true;
    })()
  )
    return true;
  return false;
};"###
    )
}

#[test]
fn verify_first_only() {
    use serde_json;

    #[derive(Serialize, TypeScriptify)]
    struct S {
        #[typescript(check = "first")]
        vals: Vec<String>,
    }
    let verify_first_only = prettier(&S::type_script_verify().unwrap());
    assert_snapshot_matches!(verify_first_only,
    @r###"export const isa_S = (obj: any): obj is S => {
  if (obj == undefined) return false;
  if (obj.vals === undefined) return false;
  {
    const val = obj.vals;
    if (!Array.isArray(val)) return false;
    for (let x of val) {
      if (!(typeof x === "string")) return false;
      break;
    }
  }
  return true;
};"###)
}
