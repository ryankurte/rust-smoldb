# smoldb

An ***extremely experimental*** abstract database interface over SQLite, designed to allow type-safe NoSQL like storage of serde compatible objects with indexing, without needing to write any SQL

This consists of a set of traits in [smoldb_traits](smoldb_traits) and a set of proc macros in [smoldb_derive](smoldb_derive), with everything conveniently re-exported in the top-level `smoldb` crate.

Check out the [example](smoldb/tests/example.rs) for a working example.

### Features

- [x] SQL Basics
  - [x] Insert
  - [x] Select
  - [x] Update
  - [x] Delete
- [ ] Extended types (as it stands, everything is strings)
  - [ ] Integers
  - [ ] Strings
  - [ ] Blobs


### TODOs

- [x] Clean up / split macros
- [x] Make `Store` impl generic over `Storage` types
- [ ] Genericise over serde encoder/decoder
- [ ] Fix Index types to use `&str` and `&[u8]` in place of `String` and `Vec<u8>` respectively

## Usage

First, define a type that is serializable and derives from `Smoldb`.
The `#[index]` macro specifies that you would like to be able to search
for objects using this index

```rust

#[derive(Clone, Debug, PartialEq, Smoldb, Serialize, Deserialize)]
pub struct User {
  #[index]
  pub id: isize,

  #[index]
  pub name: String,

  pub email: String,

  pub description: String,
}
```

You can then interact with objects using the `Store` traits. You may notice that some methods require the fully qualified trait syntax to provide type information.


