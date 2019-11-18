//! Contains various items related to errors inside of TinyDB.

/// An error enum for the possible faliure states of [crate::Database::query_item]
/// but relating directly to querying.
#[derive(Debug)]
pub enum QueryError {
    /// When the given "database" ([crate::Database]) is not actually a [crate::Database].
    NotADatabase,

    /// An error was returned from the database itself.
    DatabaseError(DatabaseError),

    /// The database does not contain the query searched for.
    ItemNotFound,

    /// When the schema given to search is not valid in terms of the [crate::Database]
    /// passed in.
    TypeNotSchema,
}

/// An error enum for the possible faliure states of the [crate::Database] structure.
#[derive(Debug)]
pub enum DatabaseError {
    /// When the item queried for was not found
    ItemNotFound,

    /// A duplicate value was found when adding to the database with
    /// [crate::Database::strict_dupes] allowed.
    DupeFound,
    /// When [crate::Database::save_path] is required but is not found. This commonly
    /// happens when loading or dumping a database with [crate::Database::save_path]
    /// being [Option::None].
    SavePathRequired,

    /// Misc [std::io::Error] that could not be properly handled.
    IOError(std::io::Error),

    /// When the database could not be found. This is typically raised inside of
    /// [crate::Database::from] when it tries to retrieve the path to the database.
    DatabaseNotFound,
}
