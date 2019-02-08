
#![allow(unused_imports)]

#[macro_use]
extern crate typescript_definitions_derive;
pub use typescript_definitions_derive::*;


pub trait TypeScriptifyTrait {
    fn type_script_ify() -> String;
}
