extern crate rusqlite;

use super::super::{Datastore, Transaction, VertexQuery, EdgeQuery, QueryTypeConverter};
use models;
use errors::Error;
use util::{generate_random_secret, get_salted_hash, parent_uuid, child_uuid};
use rusqlite::Connection;
use super::util::connect_sqlite;
use super::schema;

/// A datastore that is backed by a sqlite database.
#[derive(Clone, Debug)]
pub struct SQLiteDatastore {
    db_path: String,
    connection: Connection,
}

impl SQLiteDatastore {
    pub fn new_from_memory() -> SQLiteDatastore {
        SQLiteDatastore {
            db_path: "",
            connection: connect_sqlite(&"")
        }
    }

    pub fn new_from_file(db_path: String) -> SQLiteDatastore {
        SQLiteDatastore {
            db_path: db_path,
            connection: connect_sqlite(&db_path)
        }
    }

    pub fn create_schema(db_path: &str) -> Result<()> {
        self.connection.execute_batch(schema::SCHEMA)
    }
}

impl Datastore<SQLiteDatastore> for SQLiteDatastore {
    fn has_account(&self, account_id: Uuid) -> Result<bool, Error> {
        let results = self.connection.query("SELECT 1 FROM accounts WHERE id=?", &[&account_id])?;
        Ok(!results.is_empty())
    }

    fn create_account(&self) -> Result<(Uuid, String), Error> {
        let id = parent_uuid();
        let salt = generate_random_secret();
        let secret = generate_random_secret();
        let hash = get_salted_hash(&salt[..], Some(&self.secret[..]), &secret[..]);
        let conn = self.connection;
        
        conn.execute("
            INSERT INTO accounts(id, salt, api_secret_hash)
            VALUES (?, ?, ?)
            ", &[&id, &salt, &hash]
        )?;

        Ok((id, secret))
    }

    fn delete_account(&self, account_id: Uuid) -> Result<(), Error> {
        let conn = self.connection;
        
        match conn.execute("DELETE FROM accounts WHERE id=?", &[&account_id]) {
            Ok(updated) => Ok(()),
            Err(err) => Err(Error::AccountNotFound)
        }
    }

    fn auth(&self, account_id: Uuid, secret: String) -> Result<bool, Error> {
        let conn = self.connection;

        let get_salt_results = conn.query("SELECT salt, api_secret_hash FROM accounts WHERE id=?", &[&account_id])?;

        for row in &get_salt_results {
            let salt: String = row.get(0);
            let expected_hash: String = row.get(1);
            let actual_hash = get_salted_hash(&salt[..], Some(&self.secret[..]), &secret[..]);
            return Ok(expected_hash == actual_hash);
        }

        // Calculate the hash anyways to prevent timing attacks
        get_salted_hash("", Some(&self.secret[..]), &secret[..]);
        Ok(false)
    }

    fn transaction(&self, account_id: Uuid) -> Result<SQLiteTransaction, Error> {
        let conn = self.connection;
        let trans = SQLiteTransaction::new(conn, account_id, self.secure_uuids)?;
        Ok(trans)
    }
}


/// A postgres-backed datastore transaction.
#[derive(Debug)]
pub struct SQLiteTransaction {
}