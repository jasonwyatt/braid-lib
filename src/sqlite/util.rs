extern crate rusqlite;

use rusqlite::Connection;

pub fn connect_sqlite(db_path: &str) -> Connection {
    match db_path {
        "" => Connection::open_in_memory(),
        _ => Connection::open(db_path)
    }.expect("Could not open database.")
}