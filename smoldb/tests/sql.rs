

extern crate smoldb;
use smoldb::*;


#[derive(Clone, Debug, PartialEq, Smoldb)]
pub struct Example {
    #[index]
    pub id: i64,

    #[index]
    pub name: String,

    pub description: String,
}

// SQL query generation tests

#[test]
fn sql_create() {
    assert_eq!(&Example::sql_create("test"), "CREATE TABLE test (id VARCHAR NOT NULL, name VARCHAR NOT NULL, __object BLOB NOT NULL);");
}

#[test]
fn sql_insert() {
    assert_eq!(&Example::sql_insert("test"), "INSERT INTO test (id, name, __object) VALUES (?1, ?2, ?3);");
}

#[test]
fn sql_select() {
    assert_eq!(&Example::sql_select("test", &[ExampleIndicies::Name("John Doe".to_string())]), "SELECT __object FROM test WHERE name = ?;");
}

#[test]
fn sql_update() {
    assert_eq!(&Example::sql_update("test", &[ExampleIndicies::Name("John Doe".to_string())]), "UPDATE test SET id = ?1, name = ?2, __object = ?3 WHERE name = ?;");
}

#[test]
fn sql_delete() {
    assert_eq!(&Example::sql_delete("test", &[ExampleIndicies::Name("John Doe".to_string())]), "DELETE FROM test WHERE name = ?;");
}

#[test]
fn index_id() {
    let i = ExampleIndicies::Id(16);

    assert_eq!(i.name(), "id");
    assert_eq!(i.value(), ToSqlOutput::Owned(Value::Integer(16)));
}

#[test]
fn index_name() {
    let i = ExampleIndicies::Name("abc".to_string());

    assert_eq!(i.name(), "name");
    assert_eq!(i.value(), ToSqlOutput::Owned(Value::Text("abc".to_string())));
}