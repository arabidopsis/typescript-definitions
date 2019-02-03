# wasm-typescript-definition2

Exports serde-serializable structs and enums to Typescript definitions when used with wasm-bindgen.

```rust
#[derive(Serialize, TypescriptDefinition)]
#[serde(tag = "tag", content = "fields")]
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
```

With the patched version of wasm-bindgen that supports typescript_custom_section (TODO), this will output in your `.d.ts` definition file:

```typescript
export type Enum = 
    | {"tag": "V1", "fields": { "Foo": boolean } }
    | {"tag": "V2", "fields": { "Bar": number, "Baz": number } }
    | {"tag": "V3", "fields": { "Quux": string } }
    ;
```

## Using `wasm-typescript-definition2`

In your crate create a lib target in `Cargo.toml` pointing
you your "interfaces"

```toml
[lib]
name="wasm"
path = "src/interface.rs"
crate-type = ["cdylib"]


[dependencies]
wasm-typescript-definition2 = { version="0.1.0",  path = "../wasm-typescript-definition" }
wasm-bindgen = "0.2"
serde = "1"
serde_derive = "1"

```

Then you can run

```bash
cargo +nightly build --target wasm32-unknown-unknown
mkdir pkg
wasm-bindgen target/wasm32-unknown-unknown/debug/wasm.wasm --typescript --out-dir pkg/
cat pkg/wasm.d.ts
```
If you don't have these tool then [see here](https://rustwasm.github.io/wasm-bindgen/whirlwind-tour/basic-usage.html):

```bash
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo +nightly install wasm-bindgen-cli
```

or use wasm-pack (the typscript library will be in `pkg/wasm.d.ts`)

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
wasm-pack build
```

## Using `type_script_ify`

You can ignore wasm and derive using `TypeScriptify` as long as you have the following Trait
in scope:

```rust
// interface.rs
// wasm_bindgen not needed
// use::wasm_bindgen::prelude::*;
use::serde_derive::{Serialize};
use::wasm_typescript_definition2::{TypeScriptify};
// *you* have to provide this Trait because, currently, rust proc-macro crates can't
// export any public Traits etc... sorry.
pub trait TypeScriptifyTrait {
    fn type_script_ify() -> &'static str;
}
#[derive(Serialize, TypeScriptify)]
pub struct MyStruct {
    v : i32,
}
```
Then in `main.rs` say you can generate your own typscript specification using `Struct::type_script_ify()`:

```rust
mod interface;
// need to pull in trait
use self::interface::{TypeScriptifyTrait};

fn main() {
    println!("{}", interface::MyStruct::type_script_ify());
    // prints "export type MyStruct = { "v": number };"
}
```

## Serde Internally or Adjacently tagged Enums

[Docs](https://serde.rs/enum-representations.html#internally-tagged).

This crate understands `#[serde(tag="type")]` and `#[serde(tag="tag", content="fields")]`
attributes but only for Struct variants. 

It doesn't do Untagged or Externally tagged enums but defaults
to `#[serde(tag="kind")]` (Internal). 

The default for NewTypes and Tuple types is
`#[serde(tag="kind", content="fields")]` (Adjacent).

## Credit

see http://timryan.org/2019/01/22/exporting-serde-types-to-typescript.html

Forked from [`wasm-typescript-definition` by @tcr](https://github.com/tcr/wasm-typescript-definition?files=1)
Which was forked from [`rust-serde-schema` by @srijs](https://github.com/srijs/rust-serde-schema?files=1).

`type_script_ify` idea from [`typescriptify` by @n3phtys](https://github.com/n3phtys/typescriptify)

## License

MIT or Apache-2.0, at your option.
