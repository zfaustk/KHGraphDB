# Transaction

The arena copies. Writes hit the live graph. Drop
puts the clone back unless commit dropped the
snapshot.

```
let mut g = Graph::new();
{
    let mut tx = Tx::begin(&mut g);
    query::run(tx.graph(), "CREATE (n:Person {name:'Ada'})");
    tx.commit();
}
```

Leaving the block without commit rolls back.

khg:

```
:begin
CREATE (a:Person {name:'Ada'})
:rollback
```

`.use` is refused while a transaction is open.
There is no lock. One process, one clone.

Store is the durable tx: commit captures the
arena onto KHL1 and `sync_data`. Rollback is
not on the log. See `docs/store.md`.

