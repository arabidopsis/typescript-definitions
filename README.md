# typescript-definitions

Exports serde-serializable structs and enums to Typescript definitions.

## Motivation

Now that rust 2018 has landed
there is no question that people should be using rust to write server applications (what are you thinking!).
But generating wasm from rust code to run in the browser is currently much too bleeding edge. 

Since javascript will be dominant on the client for the forseeable future there remains the
problem of communicating with your javascript from your rust server.

Fundamental to this is to keep the datatypes on either side of the connection (http/websocket) in sync.

Typescript is an incremental typing system for javascript that is as almost(!) as tricked as rust... so
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

**NB**: Please note these macros - by default - work *only for the debug build* since they pollute the
code with strings and methods all of which are proabably not useful in any release (Since
you are only using them to extract information about your current types from your *code*). In
release builds they become no-ops. This means that there is *no cost* to your *release* exes/libs
or your users by using these macros. Zero cost abstraction indeed. Beautiful.

See [features](#features) below if you really want them in your release build.

There is a very small example in the repository that [works for me (TM)](https://bitbucket.org/athaliana/typescript-definitions/src/master/example/) if you want to get started
on your own.

This crate only exports two derive macros: `TypescriptDefinition` and `TypeScriptify`.

In your crate create a lib target in `Cargo.toml` pointing
to your "interfaces"

```toml
[lib]
name = "mywasm" # whatever... you decide
path = "src/interface.rs"
crate-type = ["cdylib"]


[dependencies]
typescript-definitions = "0.1"
wasm-bindgen = "0.2"
serde = "1"
serde_derive = "1"

```

Then you can run (see [here](#using-type_script_ify) if you don't want to go near WASM):

```bash
cargo +nightly build --target wasm32-unknown-unknown
mkdir pkg
wasm-bindgen target/wasm32-unknown-unknown/debug/mywasm.wasm --typescript --out-dir pkg/
cat pkg/mywasm.d.ts
```
If you don't have these tools then [see here](https://rustwasm.github.io/wasm-bindgen/whirlwind-tour/basic-usage.html)
(You might also need to get [rustup](https://rustup.rs) first):
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

You can ignore WASM *totally* and derive using `TypeScriptify`:

```rust
// interface.rs
extern crate serde_derive;
extern crate typescript_definitions;
// wasm_bindgen not needed
// use::wasm_bindgen::prelude::*;
use::serde_derive::Serialize;
#[allow(unused)]
use::typescript_definitions::{TypeScriptify, TypeScriptifyTrait};

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
use::typescript_definitions::TypeScriptifyTrait;

fn main() {
    println!("{}", interface::MyStruct::type_script_ify());
    // prints "export type MyStruct = { v: number };"
}
```
Use the cfg macro To protect  any use of `type_script_ify()`

```
if cfg!(any(debug_assertions, feature="export-typescript") {
    let s = A::type_script_ify();
}
```

If you have a generic struct such as:

```rust
#[derive(Serialize, TypeScriptify)]
pub struct Value<T> {
    value: T
}
```

then you need to choose a concrete type to generate the typescript: `Value<i32>::type_script_ify()`. The concrete type
doesn't matter as long as it obeys rust restrictions; the output will still be generic `export type Value<T> { value: T }`.

Currently type bounds are discarded.

So basically with `TypeScriptify` *you* have to create some binary that, via `println!` or similar statements, will
cough up a typescript library file. I guess you have more control here... at the expense of complicating
your `Cargo.toml` file and your code.


### Features

As we said before `typescript-descriptions` macros pollute your code with
static strings and other garbage. Hence, by default, they only *work* in debug mode.


If you actually want `T::type_script_ify()` (for TypeScriptify) available in your
release code then change your `Cargo.toml` file to:

```toml
[dependencies.typescript-definitions]
version = "0.1"
features = ["export-typescript"]

## OR

typescript-definitions = { version="0.1",  features=["export-typescript"]  }
```

AFAIK the strings generated by TypescriptDescription don't survive the invocation
of `wasm-bindgen` even in debug mode. So your *.wasm files are clean. You still need
to add `--features=export-typescript` to generate anything in release mode though.


## Serde Internally or Adjacently tagged Enums

See Serde [Docs](https://serde.rs/enum-representations.html#internally-tagged).

This crate understands `#[serde(tag="type")]` and `#[serde(tag="tag", content="fields")]`
attributes but only for Struct variants. 

It doesn't do Untagged or Externally tagged enums but defaults
to `#[serde(tag="kind")]` (Internal). 

The default for NewTypes and Tuple types is
`#[serde(tag="kind", content="fields")]` (Adjacent).

## Problems

Oh yes there are problems....

Currently `typescript-descriptions` will not fail (AFAIK) even for
structs and enums with function pointers `fn(a:A, b: B) -> C` (generates typescript lambda `(a:A, b:B) => C`)
and closures `Fn(A,B) -> C` (generates `(A,B) => C`). These make no sense in the current 
context (data types, json serialization) so this might be considered a bug.
Watchout!

This might change if use cases show that an error would be better.

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

Also Trait bounds are stripped out for typescript; you can't serialze Traits! However...

If using `type_script_ify` then anything other than simple trait bounds will fail to compile. (because
the `impl<...> TypeScriptify for T<...> {}` that is automatically generated by `typescript-descriptions` will be garbled).

* no `where` clauses.
* no generic Traits.

Use `WASM` instead.

This might be relaxed in future.

----

The follwing types are rendered as:

* `Option<T>` => `T | undefined`
* `HashMap<K,V>` => `{ [key:K]:V }` (same for `BTreeMap`)
* `HashSet<V>` => `V[]` (same for `BTreeSet`)

An `enum` that is all Unit types such as

```rust
enum Color {
    Red,
    Green,
    Blue
}
```
is rendered as:

```typescript
enum Color {
    Red = "Red",
    Green ="Green",
    Blue = "Blue"
}
```

because serde_json will render `Color::Red` as the string "Red" instead of `Color.Red`
(because JSON).

Serde always seems to render `Result` (in json) as `{"Ok": T } | {"Err": E}` i.e as "External"
so we do too.




Formatting is rubbish and won't pass tslint. This is due to the quote! crate taking control of the output
token stream. I don't know what it does with whitespace for example... (is whitespace a token in rust?).
Anyhow... this crate applies a few bandaid regex patches to pretty things up.


We are not as clever as serde or the compiler in determining the actual type. For example this won't "work":

```rust
use std::borrow::Cow as Pig;

#[derive(TypeScriptify)]
struct S<'a> {
    pig: Pig<'a, str>,
```

gives `export type S = { pig : Pig<string> }` instead of `export type S = { pig : string }`

At a certain point `typescript-definitions` just *assumes* that the token identifier `i32` (say)
*is* really the rust signed 32 bit integer and not some crazy renamed struct in your code!

Complex paths are ignored `std::borrow::Cow` and `mycrate::mod::Cow` are the same to us. We're
not going to reimplement the compiler to find out if they are *actually* different. A Cow is
always "Clone on write".

We can't reasonably obey serde attributes like "flatten" since we would need
to find the *actual* Struct object (from somewhere) and query its fields.

## TODO

Generate a typescript verifier for each type (maybe).

```typescript
export verify_A<T>(obj: any): boolean {/*... */ }
// *or*
export verify_A<T>(obj: any): {Ok: A<T>} | {Err: string} {/* ... */}
// *or* using guards https://www.typescriptlang.org/docs/handbook/advanced-types.html
export is_A<T>(obj: any): obj is A<T> { /* ... */ }
```
or something...

Then one could:

```typescript
let o : any = JSON.parse(some_string_from_the_inet);
if verify_A<number>(o) {
    return obj as A<number>
} else {
    // err....
}
```

maybe...

## Credits

For intial inspiration see http://timryan.org/2019/01/22/exporting-serde-types-to-typescript.html

Forked from [`wasm-typescript-definition` by @tcr](https://github.com/tcr/wasm-typescript-definition?files=1)
which was forked from [`rust-serde-schema` by @srijs](https://github.com/srijs/rust-serde-schema?files=1).

`type_script_ify` idea from [`typescriptify` by @n3phtys](https://github.com/n3phtys/typescriptify)

Probably some others...

## License

MIT or Apache-2.0, at your option.
