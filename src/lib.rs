use std::fs::File;
use std::io::Error;
use std::collections::HashMap;
use std::hash::Hash;

extern crate serde;
pub use serde::{ser::Serialize, de::Deserialize, de::DeserializeOwned};

extern crate bincode;

extern crate smoldb_derive;
pub use smoldb_derive::{Smoldb};

extern crate smoldb_traits;
pub use smoldb_traits::{Store, Storable, OBJECT_KEY};


