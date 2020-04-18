
extern crate serde;
pub use serde::{ser::Serialize, de::DeserializeOwned};

extern crate rusqlite;
use rusqlite::ToSql;


pub const OBJECT_KEY: &str = "__object";

/// Storable trait provides methods to support use with Store.
/// This should be auto-implemented using the `Smoldb` macro
pub trait Storable {
    type Indicies;

    /// Generate create table string
    fn sql_create(name: &str) -> String;

    /// Generate insert string
    fn sql_insert(table_name: &str) -> String;

    /// Generate select string
    fn sql_select(table_name: &str, indicies: &[Self::Indicies]) -> String;

    /// Generate delete string
    fn sql_update(table_name: &str, indicies: &[Self::Indicies]) -> String;

    /// Generate delete string
    fn sql_delete(table_name: &str, indicies: &[Self::Indicies]) -> String;

    /// Generate params from object body
    fn params<'a>(&'a self) -> Vec<Box<&'a dyn ToSql>>;
}

