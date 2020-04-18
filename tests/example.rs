

#[macro_use]
extern crate rusqlite;
use rusqlite::{Connection, ToSql, types::ToSqlOutput};

#[macro_use]
extern crate serde;

extern crate smoldb;
use smoldb::*;

#[derive(Clone, Debug, PartialEq, Smoldb, Serialize, Deserialize)]
pub struct Example {
    #[index]
    pub name: String,

    #[index]
    pub email: String,

    pub description: String,
}

const TABLE_NAME: &str = "test_table";


#[test]
fn integration() {
    // Create SQLite connection
    let conn = Connection::open_in_memory().unwrap();

    // Build new table
    Store::<Example>::create_table(&conn, TABLE_NAME).unwrap();

    let e = Example {
        name: "Jane Smith".to_string(),
        email: "jasm@abcd.org".to_string(),
        description: "Test user!".to_string(),
    };

    // Insert object into table
    conn.insert(TABLE_NAME, &e).unwrap();

    // Fetch all objects from the table
    let e1 = Store::<Example>::select(&conn, TABLE_NAME, &[]).unwrap();
    assert_eq!(&e1, &[e.clone()]);

    // Fetch objects with an index
    let e1 = Store::<Example>::select(&conn, TABLE_NAME, &[ExampleIndicies::Name("Jane Smith".to_string())]).unwrap();
    assert_eq!(&e1, &[e.clone()]);

    let e1 = Store::<Example>::select(&conn, TABLE_NAME, &[ExampleIndicies::Name("Dave".to_string())]).unwrap();
    assert_eq!(&e1, &[]);
}


#[test]
fn sql_create() {
    assert_eq!(&Example::sql_create("test"), "CREATE TABLE test (name VARCHAR NOT NULL, email VARCHAR NOT NULL, __object BLOB NOT NULL);");
}

#[test]
fn sql_insert() {
    assert_eq!(&Example::sql_insert("test"), "INSERT INTO test (name, email, __object) VALUES (?1, ?2, ?3);");
}

#[test]
fn sql_select() {
    assert_eq!(&Example::sql_select("test", &[ExampleIndicies::Name("John Doe".to_string())]), "SELECT __object FROM test WHERE name = ?;");
}