use proc_macro2;
use regex;

// NOTE:
// #[cfg(feature="test")] so that test functions
// within derive_typescript_definition are compiled...

// can't get access to typescript_definitions::patch!?
// so we do our own.
pub fn patch(ts: proc_macro2::TokenStream) -> String {
    let s = ts.to_string(); // why do I have to do this?
    s.replace("[ ]", "[  ]").replace("{ }", "{  }")
}
// for type_script_ify
pub fn patcht(ts: proc_macro2::TokenStream) -> String {
    let s = ts.to_string();
    s.replace(" : ", ": ")
        .replace(" ;", ";")
        .replace(" < ", "<")
        .replace(" > ", ">")
        .replace("{ }", "{}")
        .replace("[ ]", "[]")
        .replace(" ;", ";")
        .replace(">=", "> =")
}
pub fn normalize(s : String) -> String {
    let space : regex::Regex = regex::Regex::new(r"\s+").unwrap();
    space.replace_all(&s, " ").to_string()
}
