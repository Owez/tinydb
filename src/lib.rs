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
//!
//! # Essential operations
//!
//! - Create: [Database::new]
//! - Create from file: [Database::from]   
//! - Query: [Database::query_item]
//! - Update: [Database::update_item]
//! - Delete: [Database::remove_item]
//! - Get all: [Database::read_db]
//! - Dump: [Database::dump_db]

use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::hash;
use std::path::PathBuf;

/// An error enum for the possible faliure states of [Database::query_item]
/// but relating directly to querying.
#[derive(Debug)]
pub enum QueryError {
    /// When the given "database" ([Database]) is not actually a [Database].
    NotADatabase,

    /// An error was returned from the database itself.
    DatabaseError(DatabaseError),

    /// When the schema given to search is not valid in terms of the [Database]
    /// passed in.
    TypeNotSchema,
}

/// An error enum for the possible faliure states of the [Database] structure.
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

    /// Retrives a dump file from the path given and loads it.
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

    /// Reads all items from database and returns the native HashSet used.
    pub fn read_db(&self) -> &HashSet<T> {
        unimplemented!();
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

    /// Query the database for a specific item.
    ///
    /// # Syntax
    ///
    /// ```none
    /// self.query_item(|[p]| [p].[field], [query]);
    /// ```
    ///
    /// - `[p]` The closure (Will be whatever the database currently is saving as a schema).
    /// - `[field]` The exact field of `p`. If the database doesn't contain structures, don't add the `.[field]`.
    /// - `[query]` Item to query for. This is a generic and can be of any reasonable type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::Serialize;
    /// use tinydb::Database;
    ///
    /// #[derive(Eq, Hash, Serialize)]
    /// struct TestStruct {
    ///     my_age: i32
    /// }
    ///
    /// fn main() {
    ///     let my_struct = TestStruct { my_age: 329 };
    ///     let mut my_db = Database::new(String::from("query_test"), None, false);
    ///
    ///     my_db.add_item(&my_struct);
    ///
    ///     let results = my_db.query_item(|f| &f.my_age, 329);
    ///
    ///     assert_eq!(results, Ok(&my_struct));
    /// }
    /// ```
    pub fn query_item<Q>(&self, value: impl Fn(&T) -> &Q, query: Q) -> Result<&T, QueryError> {
        unimplemented!();
    }

    /// Opens the path given in [Database::save_path] or returns a [DatabaseError].
    fn open_db_path(&self) -> Result<File, DatabaseError> {
        let definate_path = self.smart_path_get();

        if definate_path.exists() {
            io_to_dberror(std::fs::remove_file(&definate_path))?;
        }

        io_to_dberror(File::create(&definate_path))
    }

    /// Automatically allocates a path for the database if [Database::save_path]
    /// is not provided. If it is, this function will simply return it.
    fn smart_path_get(&self) -> PathBuf {
        if self.save_path.is_none() {
            return PathBuf::from(format!("{}.tinydb", self.label));
        }

        PathBuf::from(self.save_path.as_ref().unwrap())
    }
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
    #[derive(Clone, Hash, Eq, PartialEq, Debug, Serialize)]
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
            Some(PathBuf::from("test.tinydb")),
            true,
        );

        my_db.add_item(DemoStruct {
            name: String::from("Xander"),
            age: 33,
        })?;
        my_db.add_item(DemoStruct {
            name: String::from("John"),
            age: 54,
        })?;

        my_db.dump_db()?;

        Ok(())
    }
    /// Tests [Database::query_item]
    #[test]
    fn query_item_db() {
        let mut my_db = Database::new(
            String::from("Query test"),
            Some(PathBuf::from("test.tinydb")),
            true,
        );

        my_db.add_item(DemoStruct {
            name: String::from("Rimmer"),
            age: 5,
        }).unwrap();
        my_db.add_item(DemoStruct {
            name: String::from("Cat"),
            age: 10,
        }).unwrap();
        my_db.add_item(DemoStruct {
            name: String::from("Kryten"),
            age: 3000,
        }).unwrap();
        my_db.add_item(DemoStruct {
            name: String::from("Lister"),
            age: 62,
        }).unwrap();

        assert_eq!(
            my_db.query_item(|f| &f.age, 62).unwrap(),
            &DemoStruct {
                name: String::from("Lister"),
                age: 62,
            }
        ); // Finds "Lister" by searching [DemoStruct::age]
        assert_eq!(
            my_db.query_item(|f| &f.name, String::from("Cat")).unwrap(),
            &DemoStruct {
                name: String::from("Kryten"),
                age: 3000,
            }
        ); // Finds "Cat" by searching [DemoStruct::name]
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
