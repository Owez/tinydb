//! Contains various items related to errors inside of TinyDB.

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

    /// When the given database name to an assumption-making function like
    /// [crate::Database::auto_from] does not have a valid file stem or could not
    /// convert from an [std::ffi::OsString] to a [String].
    BadDbName,
}

impl From<std::io::Error> for DatabaseError {
    fn from(e: std::io::Error) -> Self {
        DatabaseError::IOError(e)
    }
}
