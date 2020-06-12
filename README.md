# TinyDB

**Note not affiliated with the python [tinydb](https://tinydb.readthedocs.io/en/latest/), accidental naming error**

TinyDB or `tinydb` is a small-footprint, superfast database designed to be used in-memory and easily dumped/retrieved from a file when it's time to save. ✨

This database aims to provide an easy frontend to an efficiant in-memory database (that can also be dumped to a file). It purposefully disallows duplicate items to be sorted due to constraints with hash tables.

- [Documentation](https://docs.rs/tinydb)
- [Crates.io](https://crates.io/crates/tinydb)

## Example 🚀

A simple example of adding a structure then querying for it:

```rust
use serde::{Serialize, Deserialize};
use tinydb::Database;

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Clone)]
struct ExampleStruct {
    my_age: i32
}

fn main() {
    let my_struct = ExampleStruct { my_age: 329 };
    let mut my_db = Database::new(String::from("query_test"), None, false);

    my_db.add_item(my_struct.clone());

    let results = my_db.query_item(|s: &ExampleStruct| &s.my_age, 329);

    assert_eq!(results.unwrap(), &my_struct);
}
```
