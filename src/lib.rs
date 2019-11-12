//! # About
//!
//! A small-footprint database implamentation, originally designed for the
//! [Zeno](https://gitlab.com/zeno-src/zeno) code editor.
//!
//! Under the surface, tinydb uses a [HashSet]-based table that works in a similar
//! fashion to SQL-like/Grid based databases. There is soon planned to be a binary
//! export option for the database, allowing for embedded databases.

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
        } else {
            Err(DatabaseError::ItemNotFound)
        }
    }

    /// Query the database for a specific item.
    ///
    /// Behind-the-scenes, this is a simple wrapper around [HashSet.get].
    pub fn query_item(&mut self, item: T) -> Option<&T> {
        self.items.get(&item)
    }

    /// Loads all into database from a `.tinydb` file and **erases any current
    /// in-memory data**.
    ///
    /// Please ensure that [Database::save_path] is valid before using this.
    pub fn load_db(&self) {
        unimplemented!();
    }

    /// Dumps database to a new `.tinydb` file.
    ///
    /// Please ensure that [Database::save_path] is valid before using this.
    pub fn dump_db(&self) {
        unimplemented!();
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

        my_db.add_item(DemoStruct {
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
