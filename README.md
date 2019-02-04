# typescript-definitions

Exports serde-serializable structs and enums to Typescript definitions.

## Motivation

Now that rust 2018 has landed
there is no question that people should be using rust to write server applications (what are you thinking!).
But currently generating wasm from rust code to run in the browser is too bleeding edge. 
Since javascript will be dominant on the client for the forseeable future there remains the
problem of communicating with your javascript from your rust server.

Fundamental to this is to keep the datatypes on either side of the connection (http/websocket) in sync.

Typescript is an incremental typing system for javascript that is as tricked as rust (almost!) so
why not create a typescript definition library based on your rust code?

Please see [Credits](#credits).

example:

```rust
extern crate serde_derive;
extern crate typescript_definitions;
extern crate wasm_bindgen;

use::wasm_bindgen::prelude::*;
use::serde_derive::Serialize;
use::typescript_definitions::TypescriptDefinition;

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

With wasm-bindgen this will output in your `.d.ts` definition file:

```typescript
export type Enum = 
      {tag: "V1", fields: { Foo: boolean } }
    | {tag: "V2", fields: { Bar: number, Baz: number } }
    | {tag: "V3", fields: { Quux: string } }
    ;
```

## Using `typescript-definitions`

In your crate create a lib target in `Cargo.toml` pointing
to your "interfaces"

```toml
[lib]
name = "mywasm" # whatever... you decide
path = "src/interface.rs"
crate-type = ["cdylib"]


[dependencies]
typescript-definitions =version="0.1.0"
wasm-bindgen = "0.2"
serde = "1"
serde_derive = "1"

```

Then you can run

```bash
cargo +nightly build --target wasm32-unknown-unknown
mkdir pkg
wasm-bindgen target/wasm32-unknown-unknown/debug/mywasm.wasm --typescript --out-dir pkg/
cat pkg/mywasm.d.ts
```
If you don't have these tools then [see here](https://rustwasm.github.io/wasm-bindgen/whirlwind-tour/basic-usage.html):

```bash
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo +nightly install wasm-bindgen-cli
```

or use wasm-pack (the typescript library will be in `pkg/mywasm.d.ts`)

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
wasm-pack build
cat pkg/mywasm.d.ts
```

## Using `type_script_ify`

You can ignore WASM totally and derive using `TypeScriptify` as long as you have the following Trait
in scope:

```rust
// interface.rs
extern crate serde_derive;
extern crate typescript_definitions;
// wasm_bindgen not needed
// use::wasm_bindgen::prelude::*;
use::serde_derive::Serialize;
use::typescript_definitions::TypeScriptify;
// *you* have to provide this Trait because, currently, rust proc-macro crates can't
// export any public Traits etc... sorry about that.
pub trait TypeScriptifyTrait {
    fn type_script_ify() -> &'static str;
}
#[derive(Serialize, TypeScriptify)]
pub struct MyStruct {
    v : i32,
}
```
Then in `main.rs` (say) you can generate your own typescript specification using `Struct::type_script_ify()`:

```rust
mod interface;
// need to pull in trait
use self::interface::TypeScriptifyTrait;

fn main() {
    println!("{}", interface::MyStruct::type_script_ify());
    // prints "export type MyStruct = { v: number };"
}
```

## Serde Internally or Adjacently tagged Enums

See Serde [Docs](https://serde.rs/enum-representations.html#internally-tagged).

This crate understands `#[serde(tag="type")]` and `#[serde(tag="tag", content="fields")]`
attributes but only for Struct variants. 

It doesn't do Untagged or Externally tagged enums but defaults
to `#[serde(tag="kind")]` (Internal). 

The default for NewTypes and Tuple types is
`#[serde(tag="kind", content="fields")]` (Adjacent).

## Problems

Currently `wasm-typescript-description2` will not fail (AFAIK) even for
structs and enums with function types `Fn(A,B) -> C` (generates `C`). These make no sense in the current 
context (data types, json serialization) so this might be considered a bug.
Watchout!

This might change if use cases show that an error would be better.

Two of the more common Enums are translated differently from Tagged types

* `Option<T>` => `T | null`
* `Result<T,E>` => `T | E` instead of {tag:'T', ...} | {tag:'E', ...}

Deeply nested types might not translate correctly since currenly no attention it taken
with precedence of the '|' separator. For example it is possible
that `Result<Option<T>,E>` will become `T|null|E` instead of `(T|null)|E`. But this
flattening seems better for typescript.

If you reference another type in a struct e.g.

```rust
    #[derive(Serialize)]
    struct B<T> {q: T}
    
    #[derive(Serialize, TypescriptDefinition)]
    struct A {
        x : f64, /* simple */
        b: B<f64>,
    }
```
then this will "work" (producing `export type A = { x: number ,b: B<number> })`) but B will be opaque to
javascript unless B is *also* `#[derive(TypescriptDefinition)]`. 

Currently there is no help for this.

Formatting is rubbish and won't pass tslint. This is due to the quote! crate taking control of the output
token stream. I don't know what it does with whitespace for example... (is whitespace a token in rust?).

Possibly add token guards `___put_newline_here___` at points where I want a newline and then stripping them out
with a regex after the stream is turned into a string.

We are not as clever as serde in determining the actual type. For example this won't "work":

```rust
use std::borrow::Cow as Pig;

#[derive(TypeScriptify)]
struct S<'a> {
    pig: Pig<'a, str>,
```

gives `export type S = { pig : Pig<string> }` instead of `export type S = { pig : string }`

We can't reasonably obey serde attributes like "flatten" since we would need
to find the *actual* Struct object (from somewhere) and query its fields.



## <a name="credits"></a> Credits

see http://timryan.org/2019/01/22/exporting-serde-types-to-typescript.html

Forked from [`wasm-typescript-definition` by @tcr](https://github.com/tcr/wasm-typescript-definition?files=1)
which was forked from [`rust-serde-schema` by @srijs](https://github.com/srijs/rust-serde-schema?files=1).

`type_script_ify` idea from [`typescriptify` by @n3phtys](https://github.com/n3phtys/typescriptify)

Probably some others...

## License

MIT or Apache-2.0, at your option.
