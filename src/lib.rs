// use std::collections::HashSet;

// #[derive(Hash, Eq, PartialEq, Debug)]
// struct MySchema {
//     name: String,
//     other: Option<i32>,
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn boop() {
//         let mut db = HashSet::new();

//         db.insert(MySchema {
//             name: String::from("Hi"),
//             other: None,
//         });
//     }
// }

use std::collections::HashSet;
use std::hash;
use std::path::PathBuf;

#[derive(Debug)]
pub enum DatabaseError {
    /// When the item queried for was not found
    ItemNotFound,

    /// A duplicate value was found when adding to the database with
    /// [Database::allow_dupes] disallowed.
    DupeFound,
}

pub struct Database<T: hash::Hash + Eq> {
    pub label: String,
    pub save_path: Option<PathBuf>,
    pub allow_dupes: bool,
    items: HashSet<T>,
}

impl<T: hash::Hash + Eq> Database<T> {
    /// Creates a new database instance.
    pub fn new(label: String, save_path: Option<PathBuf>, allow_dupes: bool) -> Self {
        Database {
            label: label,
            save_path: save_path,
            allow_dupes: allow_dupes,
            items: HashSet::new(),
        }
    }

    /// Adds a new item to the in-memory database.
    pub fn add_item(&mut self, item: T) -> Result<(), DatabaseError> {
        if !self.allow_dupes {
            if self.items.contains(&item) {
                return Err(DatabaseError::DupeFound);
            }
        }

        self.items.insert(item);
        return Ok(());
    }

    /// Removes an item from the database or commonly returns
    /// [DatabaseError::ItemNotFound].
    pub fn remove_item(&mut self, item: T) -> Result<(), DatabaseError> {
        if self.items.remove(&item) {
            Ok(())
        }
        else {
            Err(DatabaseError::ItemNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dummy struct to use inside of tests
    #[derive(Hash, Eq, PartialEq, Debug)]
    struct DemoStruct {
        name: String,
        age: i32,
    }

    #[test]
    fn db_add() -> Result<(), DatabaseError> {
        let mut my_db = Database::new(String::from("Adding test"), None, true);

        my_db
            .add_item(DemoStruct {
                name: String::from("John"),
                age: 16,
            })?;

        Ok(())
    }

    #[test]
    fn db_remove() -> Result<(), DatabaseError> {
        let mut my_db = Database::new(String::from("Adding test"), None, true);

        let testing_struct = DemoStruct {
            name: String::from("Xander"),
            age: 33,
        };

        my_db.add_item(&testing_struct)?;
        my_db.remove_item(&testing_struct)?;

        Ok(())
    }
}
