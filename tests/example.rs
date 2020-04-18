

extern crate rusqlite;
use rusqlite::{Connection, ToSql, types::ToSqlOutput};

#[macro_use]
extern crate serde;

extern crate smoldb;
use smoldb::*;

#[derive(Clone, Debug, PartialEq, Smoldb, Serialize, Deserialize)]
pub struct User {
    //#[index]
    pub id: isize,

    #[index]
    pub name: String,

    pub email: String,

    pub description: String,
}

const TABLE_NAME: &str = "test_table";


#[test]
fn integration() {
    // Create SQLite connection
    let conn = Connection::open_in_memory().unwrap();

    // Build new table
    Store::<User>::create_table(&conn, TABLE_NAME).unwrap();

    let mut e = User {
        id: 0,
        name: "Jane Smith".to_string(),
        email: "jasm@abcd.org".to_string(),
        description: "Test user!".to_string(),
    };

    // Insert object into table
    conn.insert(TABLE_NAME, &e).unwrap();

    // Fetch all objects from the table
    let e1: Vec<User> = conn.select(TABLE_NAME, &[]).unwrap();
    assert_eq!(&e1, &[e.clone()]);

    // Fetch objects with an index
    let e1: Vec<User> = conn.select(TABLE_NAME, &[UserIndicies::Name("Jane Smith".to_string())]).unwrap();
    assert_eq!(&e1, &[e.clone()]);

    let e1: Vec<User> = conn.select(TABLE_NAME, &[UserIndicies::Name("Dave".to_string())]).unwrap();
    assert_eq!(&e1, &[]);

    // Update an object
    e.email = "asdfsfd@asd.com".to_string();
    e.description = "A replacement description".to_string();

    conn.update(TABLE_NAME, &[UserIndicies::Name("Jane Smith".to_string())], &e).unwrap();

    // Check object was updated
    let e1: Vec<User> = conn.select(TABLE_NAME, &[UserIndicies::Name("Jane Smith".to_string())]).unwrap();
    assert_eq!(&e1, &[e.clone()]);

    // Delete object
    Store::<User>::delete(&conn, TABLE_NAME, &[UserIndicies::Name("Jane Smith".to_string())]).unwrap();

    // Check no objects exist
    let e1: Vec<User> = conn.select(TABLE_NAME, &[]).unwrap();
    assert_eq!(&e1, &[]);
}
