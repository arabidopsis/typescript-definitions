# wasm-typescript-definition

Exports serde-serializable structs and enums to Typescript definitions when used with wasm-bindgen.

```typescript
#[derive(Serialize, TypescriptDefinition)]
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
    | {"tag": "V1", "fields": { "Foo": boolean, }, }
    | {"tag": "V2", "fields": { "Bar": number, "Baz": number, }, }
    | {"tag": "V3", "fields": { "Quux": string, }, }
    ;
```

## Credit


Forked from [`wasm-typescript-definition` by @tcr](https://github.com/tcr/wasm-typescript-definition?files=1)
Forked from [`rust-serde-schema` by @srijs](https://github.com/srijs/rust-serde-schema?files=1).

## License

MIT or Apache-2.0, at your option.
