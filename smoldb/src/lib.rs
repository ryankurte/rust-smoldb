

use std::fmt::Debug;

#[macro_use]
extern crate log;

extern crate serde;
pub use serde::{ser::Serialize, de::Deserialize, de::DeserializeOwned};

extern crate bincode;

#[macro_use]
extern crate rusqlite;
pub use rusqlite::{Connection, ToSql, types::ToSqlOutput, types::Value};

extern crate smoldb_derive;
pub use smoldb_derive::{Smoldb};

extern crate smoldb_traits;
pub use smoldb_traits::{Storable, OBJECT_KEY};

pub trait Store<T> {
    type Indicies;
    type Error;

    fn create_table(&self, table_name: &str) -> Result<(), Self::Error>;

    fn insert(&self, table_name: &str, t: &T) -> Result<(), Self::Error>;

    fn select(&self, table_name: &str, indicies: &[Self::Indicies]) -> Result<Vec<T>, Self::Error>;

    fn update(&self, table_name: &str, indicies: &[Self::Indicies], t: &T) -> Result<(), Self::Error>;

    fn delete(&self, table_name: &str, indicies: &[Self::Indicies]) -> Result<(), Self::Error>;
}

/// Generic store implementation for all Storable types
impl <T> Store<T> for rusqlite::Connection 
where
    T: Storable + Serialize + DeserializeOwned,
    <T as Storable>::Indicies: ToSql + Debug,
{
    type Error = rusqlite::Error;
    type Indicies = <T as Storable>::Indicies;

    /// Create a new table of the associated type with the specified name
    fn create_table(&self, table_name: &str) -> Result<(), Self::Error> {
        // Execute the query
        self.execute(&T::sql_create(table_name), params![])?;

        Ok(())
    }

    /// Insert a new object into the specified table
    fn insert(&self, table_name: &str, t: &T) -> Result<(), Self::Error> {
        // Encod object
        let encoded = bincode::serialize(t).unwrap();

        // Generate parameters
        let mut params = t.params();
        params.push(Box::new(&encoded));

        let s = T::sql_insert(table_name);

        debug!("Insert query: {}", s);

        // Execute query
        self.execute(&s, params)?;

        Ok(())
    }

    /// Select objects matching the provided indicies from the specified table
    fn select(&self, table_name: &str, indicies: &[Self::Indicies]) -> Result<Vec<T>, Self::Error> {

        let s = T::sql_select(table_name, indicies);
        debug!("Select indicies: {:?} query: {}", indicies, s);

        // Build query
        let mut query = self.prepare(&s)?;
        // Execute query with provided parameters
        let mut rows = query.query(indicies)?;

        // Parse out results
        let mut res = Vec::new();
        while let Some(r) = rows.next()? {
            let d: Vec<u8> = r.get(0)?;

            let o: T = bincode::deserialize(&d).unwrap();

            res.push(o);
        }
        
        Ok(res)
    }
    
    /// Update an object in the specified table, matching the provided indicies
    fn update(&self, table_name: &str, indicies: &[Self::Indicies], t: &T) -> Result<(), Self::Error> {
        let s = T::sql_update(table_name, indicies);
        debug!("Update indicies: {:?} query: {}", indicies, s);

        // Prepare query
        let mut query = self.prepare(&s)?;

        // Encode object
        let encoded = bincode::serialize(t).unwrap();

        // Setup parameters

        // Base object values
        let mut params = t.params();

        // Encoded object value
        params.push(Box::new(&encoded));

        // Where field matches
        for i in indicies {
            params.push(Box::new(i));
        }

        // Execute query
        query.execute(params)?;

        Ok(())
    }

    /// Delete (an) object(s) in the specified table matching the provided indicies
    fn delete(&self, table_name: &str, indicies: &[Self::Indicies]) -> Result<(), Self::Error> {
        let s = T::sql_delete(table_name, indicies);
        debug!("Delete indicies: {:?} query: {}", indicies, s);

        self.execute(&s, indicies)?;

        Ok(())
    }
}