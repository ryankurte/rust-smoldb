
extern crate serde;
pub use serde::{ser::Serialize, de::DeserializeOwned};


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
}

pub trait Store<T> {
    type Indicies;
    type Error;

    fn create_table(&self, table_name: &str) -> Result<(), Self::Error>;

    fn insert(&self, table_name: &str, t: &T) -> Result<(), Self::Error>;

    fn select(&self, table_name: &str, indicies: &[Self::Indicies]) -> Result<Vec<T>, Self::Error>;

    #[cfg(nope)]
    fn update(&self, table_name: &str, indicies: &[Self::Indicies], t: T) -> Result<(), Self::Error>;

    #[cfg(nope)]
    fn delete(&self, indicies: &[Self::Indicies]) -> Result<(), Self::Error>;
}

