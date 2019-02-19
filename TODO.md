
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


## Verification for Generics

Maybe just say no... who is going to use generic types for *data* transfer?

Still there is the possibility that a type will use a monomorphisation of a generic
struct.

Maybe a variants attribute:

```rust
#[typescript(variants(T="str + Vec<f64>"))]
struct S<T> { value: T}
```
With typescript as

```typescript
export type S<T>
| { value: string }
| { value: number[]}
| { value: T}

// OR maybe just as before
export type S<T> { value: T}

export type S_s = { value: string }
export type S_vn = { value: number[] }
export const isa_S = (a:any): a is S => isa_S_s(a) || isa_S_vn(a);
export const isa_S_s (a:any) a is S_s => {/*....*/}
export const isa_S_vn (a:any) a is S_vn => {/*....*/}
// etc...
```
Type erasure would allow one to instaniate `let v = S<number> = { value: 32 }`

### non generics using generics

This would allow structs using this generic:
```rust
struct T { value: S<Vec<f64>> }
```

to generate *automatically* a `isa_S_vn` verifier

```typescript
export type T { value: S<number[]> }
export const isa_T = (a:any) a is T => { isa_S_vn(a.value)}
```

## Totally generic arguments 

This will probably be last to have a verifier....

```rust
#[typescript(variants(T="str + Vec<f64>"))]
struct S2<T> {value: S<T> }
```
```typescript
export type S2<T> =
| { value : S<string> }
| { value : S<number[]>}
| { value: S<T>}
```

maybe some better name mangling `<Vec<Vec<f64>>>` go to `vvn` Could get
arbitrarily complex...






