
specify a Map<K,V> then we have to construct this from
what? as Map::from([k,v,....]). type Map<K,V> = { [k in K]: V}
o = JSON.parse(s)
o.a.b.c = new Map.from(o.a.b.c)
so we need the path... for each from the root
like o.a[5].c.contents.blah etc.
also for Set etc.


Grab random rust code with derive(Serialize) and see if my code works.

Problem with enums rendering as a string.

Maybe make a const enum as set of const strings e.g.

```typescript
type Enum = "a" | "b" | "c"
```


If we want to honour `#[serde(flatten)]` then this will only be possible
with `TypeScriptify`. we can laydown a

```rust
trait TypeScriptifyTrait {
    fn type_script_ify() -> String;
    fn fields() -> Vec<String>;
}
```

Then we can flatten by finding the fields for a type with `#T::<???>::fields()`


Configure code generation via `option_env!`. Yesss! (look at env_logger).


## Verification

... baby steps. 

```typescript
function verify_#ident(obj: any): obj is #ident {
    for let t in ["a", "b", ...] {
        let v = obj[t];
        if (v === undefined) return false; // change optional to a | null
        // TODO more checks

    }
    return true;
}
```

for simple types (number, string) we can do `if typeof v == "number"` etc.

might need to recurse into another verify_X function

for arrays #tp[]  Array.isArray(v) && all(for x in v verify_#tp(x))

probably want two types a "shallow" verification (see above) and a deep one.




