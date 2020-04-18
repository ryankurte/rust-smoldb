# smoldb

An ***extremely experimental*** abstract database interface over SQLite, designed to allow NoSQL like storage of serde compatible objects with indexing, without needing to write any SQL

This consists of a set of traits in [smoldb_traits](smoldb_traits) and a set of proc macros in [smoldb_derive](smoldb_derive), with everything conveniently re-exported in the top-level `smoldb` crate.

Check out the [example](tests/example.rs) for an example use.

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

