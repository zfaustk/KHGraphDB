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
