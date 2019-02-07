
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

