
specify a Map<K,V> then we have to construct this from
what? as Map::from([k,v,....]). type Map<K,V> = { [k in K]: V}
o = JSON.parse(s)
o.a.b.c = new Map.from(o.a.b.c)
so we need the path... for each from the root
like o.a[5].c.contents.blah etc.
also for Set etc.


Grab random rust code with derive(Serialize) and see if my code works.

Problem with enums rendering as a string.

Maybe make an enum as set of const strings e.g.

```typescript
type Enum = "a" | "b" | "c"
```

Maybe change QuoteT

```rust
enum QuoteT {
    Tokens(TokenStream),
    Builder(Box<Fn() -> TokenStream>)
}

impl ToTokens for QuoteT {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Tokens(t) => t.to_tokens(tokens),
            Builder(f) => f().to_tokens(tokens)
        }
    }
}
```

If we want to honour `#[serde(flatten)]` then this will only be possible
with `TypeScriptify`. we can laydown a

trait TypeScriptifyTrait {
    fn type_script_ify() -> String;
    fn 
}

