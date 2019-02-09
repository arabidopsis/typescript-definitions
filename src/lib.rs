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

/// # TypeScriptifyTrait
///
/// Used with the TypeScriptify proc_macro
///
/// please see documentation at [crates.io](https://crates.io/crates/typescript-definitions)
///
pub trait TypeScriptifyTrait {
    fn type_script_ify() -> String;
    fn type_script_fields() -> Option<Vec<&'static str>>;
}
