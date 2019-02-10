// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # TypeScriptifyTrait
//!
//! Used with the TypeScriptify proc_macro
//!
//! please see documentation at [crates.io](https://crates.io/crates/typescript-definitions)
//!

#![allow(unused_imports)]
#[macro_use]
pub extern crate typescript_definitions_derive;
// re-export macros
pub use typescript_definitions_derive::*;
use serde::ser::{Serializer};
/// # TypeScriptifyTrait
///
/// Used with the `TypeScriptify` proc_macro.
///
/// Please see documentation at [crates.io](https://crates.io/crates/typescript-definitions).
///
pub trait TypeScriptifyTrait {
    fn type_script_ify() -> String;
    // fn type_script_fields() -> Option<Vec<&'static str>>;
}
/// # as_byte_string
/// 
/// use `#[serde(serialize_with="typescript_definitions::as_byte_string")]`
/// on a `[u8]` or `Vec<u8>` object to  make the output type a `string` (instead of a `number[]`).
/// The encoding is a simple `\xdd` format. Or provide your own serializer:
/// `typescript-definitions` only checks the final *name* "as_byte_string" of the path.
/// e.g.
/// ```
/// # #[macro_use] extern crate typescript_definitions;
/// # #[macro_use] extern crate serde_derive;
/// 
/// #[derive(Serialize, TypeScriptify)]
/// struct S {
///     #[serde(serialize_with="typescript_definitions::as_byte_string")]
///     image : Vec<u8>
/// }
/// ```
/// outputs `export type S = { image: string };`.

pub fn as_byte_string<'a, S>(bytes: &'a [u8], serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {

    let t = bytes.iter().map(|b| format!(r"\x{:02x}", b)).collect::<Vec<_>>().join("");
    // S is FnOnce
    serializer.serialize_str(&t)

}