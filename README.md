# braid library

[![Build Status](https://travis-ci.org/braidery/braid-lib.svg?branch=master)](https://travis-ci.org/braidery/braid-lib) [rustdoc](https://braidery.github.io/apis/lib/braid)

This is the braid library. Most users can use the [server](https://github.com/braidery/braid), which provides higher-level methods for working with braid databases. Linking directly against the library would be necessary if you want to make a new datastore implementation, or if you want to work at a lower-level than the server affords.

## Pluggable datastores

Braid stores graph data in datastores. Datastores are pluggable: there is built in support for postgres and rocksdb, but you can implement a new custom datastore.

### Postgres

To use the postgres datastore, add this to your `Cargo.toml`:

```toml
[dependencies.braid]
git = "https://github.com/braidery/braid-lib"
features = ["postgres-datastore"]
```

### RocksDB

To use the rocksdb datastore, add this to your `Cargo.toml`:

```toml
[dependencies.braid]
git = "https://github.com/braidery/braid-lib"
features = ["rocksdb-datastore"]
```

### Custom datastores

To implement a custom datastore, you need to implement the [Datastore](https://braidery.github.io/apis/lib/braid/trait.Datastore.html) and [Transaction](https://braidery.github.io/apis/lib/braid/trait.Transaction.html) traits. See the [postgres](https://github.com/braidery/lib/blob/develop/src/pg/datastore.rs) and [rocksdb](https://github.com/braidery/lib/blob/develop/src/rdb/datastore.rs) datastores as examples.

To help you get off the ground faster, we've defined some standard tests that can execute against any datastore and check for common bugs and regressions. See the [postgres datastore tests](https://github.com/braidery/braid-lib/blob/develop/src/pg/tests.rs) for an example.

## Running tests

Run `./test.sh`.

## Running benchmarks

Run `./test.sh --bench`.

