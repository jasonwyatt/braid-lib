extern crate rusqlite;

use super::super::{Datastore, Transaction, VertexQuery, EdgeQuery, QueryTypeConverter};
use models;
use errors::Error;
use util::{generate_random_secret, get_salted_hash, parent_uuid, child_uuid};

/// A datastore that is backed by a sqlite database.
#[derive(Clone, Debug)]
pub struct SQLiteDatastore {
    db_path: String,
}

impl SQLiteDatastore {
    pub fn new(db_path: String) -> SQLiteDatastore {
        SQLiteDatastore {
            db_path: db_path,
        }
    }
}


/// A postgres-backed datastore transaction.
#[derive(Debug)]
pub struct SQLiteTransaction {
}