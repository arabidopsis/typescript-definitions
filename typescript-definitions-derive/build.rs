use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target = env::var("TARGET").unwrap();

    if target.contains("wasm32") {
         println!("cargo:rustc-cfg=wasm32");
    }

   
}