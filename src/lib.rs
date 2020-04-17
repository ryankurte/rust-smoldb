use std::fs::File;
use std::io::Error;
use std::collections::HashMap;

#[macro_use]
extern crate serde;

extern crate bincode;

pub const DB_VERSION: usize = 0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BlockType {
    StoreHeader = 1,
    TableHeader = 2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Options {
    pub block_size: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            block_size: 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoreHeader {
    pub store_version: usize,

    pub tables: HashMap<String, usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableHeader {
    
    pub values: HashMap<String, usize>,

    // Map to indicie blocks
    pub indicies: HashMap<String, usize>,
}

pub struct Store {
    file: File,

    tables: HashMap<String, ()>,
}  

pub enum StoreError {
    Io(std::io::Error),
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}


impl Store {
    pub fn new(path: &str) -> Result<Self, StoreError> {
        // Create new database file
        let mut file = File::create(path)?;

        let s = Self{ file, tables: HashMap::new() };

        // Write new header

        Ok(s)

    }

    pub fn load(path: &str) -> Result<Self, StoreError> {
        // Load database file
        let mut file = File::open(path)?;

        let s = Self{ file, tables: HashMap::new() };

        // Load header

        Ok(s)
    }

    /// Create a new table instance, this must be manually synced following use
    pub unsafe fn table<'a, K, V>(&'a mut self, name: &str) -> Result<Table<'a, K, V>, StoreError> {
        let t = Table{ store: self, _cache: HashMap::new() };

        // TODO: load if required

        Ok(t)
    }

    /// Update the specified table, this handles loading / creating the table and syncing on completion
    pub fn update<K, V, F: Fn(&mut Table<'_, K, V>)>(&mut self, table_name: &str, f: F) -> Result<(), StoreError> {
        unsafe {
            let mut t = self.table(table_name)?;

            f(&mut t);

            t.sync()?;
        }

        Ok(())
    }
}

pub struct Table<'a, K, V> {
    store: &'a mut Store,

    _cache: HashMap<K, V>,
}

impl <'a, K, V> Table<'a, K, V> {

    pub unsafe fn sync(mut self) -> Result<(), StoreError> {
        unimplemented!();
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
