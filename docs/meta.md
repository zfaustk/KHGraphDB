# Meta

Posting is not the page. KHM1 lists
`(type, key, value) → Addr`. Drop the
file, rebuild from the arena.

FIND reads meta. MATCH still takes one
graph. A replica honors a bookmark
before it answers; catch_up first,
or fail.

```
let bm = primary.commit()?;
replica.honor(primary.dir(), bm)?;
let m = Meta::open(replica.dir())?;
m.find("Doc", "title", "Ada");
```
