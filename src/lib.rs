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
//! [HashSet] underneath).
//!
//! # Essential operations
//!
//! Some commonly-used operations for the [Database] structure.
//!
//! | Operation                 | Implamentation          |
//! |---------------------------|-------------------------|
//! | Create database           | [Database::new]         |
//! | Create database from file | [Database::from]        |
//! | Query for item            | [Database::query_item]  |
//! | Update/replace item       | [Database::update_item] |
//! | Delete item               | [Database::remove_item] |
//! | Get all items             | [Database::read_db]     |
//! | Dump database             | [Database::dump_db]     |

#![doc(
    html_logo_url = "https://gitlab.com/Owez/tinydb/raw/master/logo.png",
    html_favicon_url = "https://gitlab.com/Owez/tinydb/raw/master/logo.png"
)]

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::hash;
use std::io::prelude::*;
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
#[derive(Serialize, Deserialize)]
pub struct Database<T: hash::Hash + Eq> {
    /// Friendly name for the database, preferibly in `slug-form-like-this` as
    /// this is the fallback path.
    pub label: String,

    /// The overwrite path to save the database as, this is reccomended otherwise
    /// it will end up as `./Hello\ There.tinydb` if [Database::label] is "Hello
    /// There".
    pub save_path: Option<PathBuf>,

    /// If the database should return an error if it tries to insert where an
    /// identical item already is. Setting this as `false` doesn't allow
    /// duplicates, it just doesn't flag an error.
    pub strict_dupes: bool,

    /// In-memory [HashSet] of all items.
    items: HashSet<T>,
}

impl<T: hash::Hash + Eq + Serialize + DeserializeOwned> Database<T> {
    /// Creates a new database instance from given parameters.
    ///
    /// - To add a first item, use [Database::add_item].
    /// - If you'd like to load a dumped database, use [Database::from].
    pub fn new(label: String, save_path: Option<PathBuf>, strict_dupes: bool) -> Self {
        Database {
            label: label,
            save_path: save_path,
            strict_dupes: strict_dupes,
            items: HashSet::new(),
        }
    }

    /// Creates a database from a `.tinydb` file.
    ///
    /// This retrives a dump file (saved database) from the path given and loads it as the [Database] structure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tinydb::Database;
    ///
    /// /// Small example structure to show.
    /// struct ExampleStruct {
    ///    data: i32
    /// }
    ///
    /// /// Makes a small testing database.
    /// fn make_db() {
    ///     let test_db = Database::new(String::from("test"), None, false);
    ///     test_db.add_item(ExampleStruct { data: 34 });
    ///     test_db.dump_db();
    /// }
    ///
    /// /// Get `test_db` defined in [make_db] and test.
    /// fn main() {
    ///     make_db();
    ///
    ///     let got_db = Database::from(
    ///         |s: &ExampleStruct| &s,
    ///         PathBuf::from("test.tinydb")
    ///     );
    ///
    ///     assert_eq!(
    ///         got_db.query_item(|s: &ExampleStruct| &s.data, 34).unwrap(),
    ///         &ExampleStruct { data: 34 }
    ///     ); // Check that the database still has added [ExampleStruct].
    /// }
    /// ```
    pub fn from(schema: impl FnOnce(&T) -> &T, path: PathBuf) -> Result<Self, DatabaseError> {
        let stream = get_stream_from_path(path)?;
        let decoded: Database<T> = bincode::deserialize(&stream[..]).unwrap();

        Ok(decoded)
    }

    /// Adds a new item to the in-memory database.
    ///
    /// If this is the first item added to the database, please ensure it's the
    /// only type you'd like to add. Due to generics, the first item you add
    /// will be set as the type to use (unless removed).
    pub fn add_item(&mut self, item: T) -> Result<(), DatabaseError> {
        if self.strict_dupes {
            if self.items.contains(&item) {
                return Err(DatabaseError::DupeFound);
            }
        }

        self.items.insert(item);
        return Ok(());
    }

    /// Essentially replaces an item with another item.
    ///
    /// [Database::query_item] can be used in conjunction to find and replace
    /// values individually if needed.
    pub fn update_item(&mut self, item: &mut T, new: T) -> Result<(), DatabaseError> {
        unimplemented!();
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

    /// Gets all items from [Database] and returns a reference to the native
    /// HashSet storage used.
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
    /// #[derive(Eq, Hash, Serialize, Deserialize)]
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
    ///     let results = my_db.query_item(|s: TestStruct| &s.my_age, 329);
    ///
    ///     assert_eq!(results, Ok(&my_struct));
    /// }
    /// ```
    pub fn query_item<Q>(&self, value: impl FnOnce(&T) -> &Q, query: Q) -> Result<&T, QueryError> {
        unimplemented!();
    }

    /// Opens the path given in [Database::save_path] (or auto-generates a path).
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

/// Reads a given path and converts it into a [Vec]<[u8]> stream.
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
    #[derive(Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
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

        my_db.add_item(testing_struct.clone())?;
        my_db.remove_item(testing_struct)?;

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

        my_db
            .add_item(DemoStruct {
                name: String::from("Rimmer"),
                age: 5,
            })
            .unwrap();
        my_db
            .add_item(DemoStruct {
                name: String::from("Cat"),
                age: 10,
            })
            .unwrap();
        my_db
            .add_item(DemoStruct {
                name: String::from("Kryten"),
                age: 3000,
            })
            .unwrap();
        my_db
            .add_item(DemoStruct {
                name: String::from("Lister"),
                age: 62,
            })
            .unwrap();

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

        let my_db: Database<DemoStruct> =
            Database::from(|s: &DemoStruct| &s, PathBuf::from("test.tinydb"))?;

        assert_eq!(my_db.label, String::from("Dumping Test"));

        Ok(())
    }
}
