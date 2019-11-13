//! # About
//!
//! A small-footprint database implamentation, originally designed for the
//! [Zeno](https://gitlab.com/zeno-src/zeno) code editor. This database does not
//! accept duplicates and will not save a second identical item.
//!
//! Under the surface, tinydb uses a [HashSet]-based table that works in a similar
//! fashion to SQL-like/Grid based databases.
//!
//! # Disclaimer
//!
//! This project is not intended to be used inside of any critical systems due to
//! the nature of dumping/recovery. If you are using this crate as a temporary and
//! in-memory only database, it should preform at a reasonable speed (as it uses
//! [HashSet] underneith).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::hash;
use std::io::prelude::*;
use std::path::PathBuf;

/// Database error enum
#[derive(Debug)]
pub enum DatabaseError {
    /// When the item queried for was not found
    ItemNotFound,

    /// A duplicate value was found when adding to the database with
    /// [Database::strict_dupes] allowed.
    DupeFound,
    /// When [Database::save_path] is required but is not found. This commonly
    /// happens when loading or dumping a database with [Database::save_path]
    /// being [Option::None].
    SavePathRequired,

    /// Misc [std::io::Error] that could not be properly handled.
    IOError(std::io::Error),

    /// When the database could not be found. This is typically raised inside of
    /// [Database::from] when it tries to retrieve the path to the database.
    DatabaseNotFound,
}

/// The primary database structure, allowing storage of a given generic.
///
/// The generic type used should primarily be structures as they resemble a
/// conventional database model and should implament [hash::Hash] and [Eq].
///
/// # Essential operations
///
/// - Create: [Database::new]
/// - Create from file: [Database::from]   
/// - Query: [Database::query_item]
/// - Update: [Database::update_item]
/// - Delete: [Database::remove_item]
/// - Get all: [Database::read_db]
/// - Dump: [Database::dump_db]
#[derive(Serialize)]
pub struct Database<T: hash::Hash + Eq + Serialize> {
    pub label: String,
    pub save_path: Option<PathBuf>,
    pub strict_dupes: bool,
    items: HashSet<T>,
}

impl<T: hash::Hash + Eq + Serialize> Database<T> {
    /// Creates a new database instance.
    pub fn new(label: String, save_path: Option<PathBuf>, strict_dupes: bool) -> Self {
        Database {
            label: label,
            save_path: save_path,
            strict_dupes: strict_dupes,
            items: HashSet::new(),
        }
    }

    /// Retrives a dump file from [path] and loads it.
    pub fn from(path: PathBuf) -> Result<Self, DatabaseError> {
        unimplemented!();
    }

    /// Adds a new item to the in-memory database.
    pub fn add_item(&mut self, item: T) -> Result<(), DatabaseError> {
        if self.strict_dupes {
            if self.items.contains(&item) {
                return Err(DatabaseError::DupeFound);
            }
        }

        self.items.insert(item);
        return Ok(());
    }

    /// Removes an item from the database.
    ///
    /// # Errors
    ///
    /// Will return [DatabaseError::ItemNotFound] if the item that is attempting
    /// to be deleted was not found.
    pub fn remove_item(&mut self, item: T) -> Result<(), DatabaseError> {
        if self.items.remove(&item) {
            Ok(())
        } else {
            Err(DatabaseError::ItemNotFound)
        }
    }

    /// Query the database for a specific item.
    pub fn query_item(&mut self, item: T) -> Option<&T> {
        self.items.get(&item)
    }

    /// Dumps/saves database to a binary file.
    ///
    /// # Saving path methods
    ///
    /// The database will usually save as `\[label\].tinydb` where `\[label\]`
    /// is the defined [Database::label] (path is reletive to where tinydb was
    /// executed).
    ///
    /// You can also overwrite this behaviour by defining a [Database::save_path]
    /// when generating the database inside of [Database::new].
    pub fn dump_db(&self) -> Result<(), DatabaseError> {
        let mut dump_file = self.open_db_path()?;
        bincode::serialize_into(&mut dump_file, self).unwrap();

        Ok(())
    }

    /// Automatically allocates a path for the database if [Database::save_path]
    /// is not provided. If it is, this function will simply return it.
    fn smart_path_get(&self) -> PathBuf {
        if self.save_path.is_none() {
            return PathBuf::from(format!("{}.tinydb", self.label));
        }

        PathBuf::from(self.save_path.as_ref().unwrap())
    }

    /// Opens the path given in [Database::save_path] or returns a [DatabaseError].
    fn open_db_path(&self) -> Result<File, DatabaseError> {
        let definate_path = self.smart_path_get();

        if definate_path.exists() {
            io_to_dberror(std::fs::remove_file(&definate_path))?;
        }

        io_to_dberror(File::create(&definate_path))
    }
}

/// Reads a given path and converts it into a &\[[u8]\] (u8 slice) stream.
fn get_stream_from_path(path: PathBuf) -> Result<Vec<u8>, DatabaseError> {
    if !path.exists() {
        return Err(DatabaseError::DatabaseNotFound);
    }

    let mut got_file = io_to_dberror(File::open(path))?;
    let mut contents: Vec<u8> = Vec::new();

    io_to_dberror(got_file.read(&mut contents))?;

    Ok(contents)
}

/// Converts a possible [std::io::Error] to a [DatabaseError].
fn io_to_dberror<T>(io_res: Result<T, std::io::Error>) -> Result<T, DatabaseError> {
    match io_res {
        Ok(x) => Ok(x),
        Err(e) => Err(DatabaseError::IOError(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dummy struct to use inside of tests
    #[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
    struct DemoStruct {
        name: String,
        age: i32,
    }

    /// Tests addition to in-memory db
    #[test]
    fn item_add() -> Result<(), DatabaseError> {
        let mut my_db = Database::new(String::from("Adding test"), None, true);

        my_db.add_item(DemoStruct {
            name: String::from("John"),
            age: 16,
        })?;

        Ok(())
    }

    /// Tests removal from in-memory db
    #[test]
    fn item_remove() -> Result<(), DatabaseError> {
        let mut my_db = Database::new(String::from("Removal test"), None, true);

        let testing_struct = DemoStruct {
            name: String::from("Xander"),
            age: 33,
        };

        my_db.add_item(&testing_struct)?;
        my_db.remove_item(&testing_struct)?;

        Ok(())
    }

    #[test]
    fn db_dump() -> Result<(), DatabaseError> {
        let mut my_db = Database::new(
            String::from("Dumping test"),
            Some(PathBuf::from("db/test.tinydb")),
            true,
        );

        for _ in 0..1 {
            let testing_struct = DemoStruct {
                name: String::from("Xander"),
                age: 33,
            };
            let other = DemoStruct {
                name: String::from("John"),
                age: 54,
            };
            my_db.add_item(testing_struct)?;
            my_db.add_item(other)?;
        }

        my_db.dump_db()?;

        Ok(())
    }

    /// Tests a [Database::from] method call
    #[test]
    fn db_from() -> Result<(), DatabaseError> {
        db_dump()?; // ensure database was dumped

        let my_db: Database<DemoStruct> = Database::from(PathBuf::from("db/test.tinydb"))?;

        assert_eq!(my_db.label, String::from("Dumping Test"));

        Ok(())
    }
}
