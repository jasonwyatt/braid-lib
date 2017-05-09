extern crate rusqlite;

use super::super::{Datastore, Transaction, VertexQuery, EdgeQuery, QueryTypeConverter};
use models;
use errors::Error;
use uuid::Uuid;
use util::{generate_random_secret, get_salted_hash, parent_uuid, child_uuid};
use rusqlite::{Connection, Statement, Result};
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

    pub fn create_schema(&self, db_path: &str) -> Result<(), ()> {
        self.connection.execute_batch(schema::SCHEMA).unwrap();
        Ok(())
    }
}

impl Datastore<SQLiteTransaction> for SQLiteDatastore {
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
    account_id: Uuid,
    conn: Box<Connection>,
    secure_uuids: bool,
}

impl SQLiteTransaction {
    fn new(conn: Connection, account_id: Uuid, secure_uuids: bool) -> Result<Self, Error> {
        conn.execute("BEGIN TRANSACTION", &[])?;

        Ok(SQLiteTransaction {
            account_id: account_id,
            conn: Box::new(conn),
            secure_uuids: secure_uuids
        })
    }

    fn prepare_vertex_query(&self, q: VertexQuery) -> Result<Statement> {
         match q {
            VertexQuery::All(start_id, limit) => {
                match start_id {
                    Some(start_id) => {
                        self.conn.prepare("SELECT id, owner_id, type FROM vertices WHERE id > ? ORDER BY id LIMIT ?")
                    },
                    None => {
                        self.conn.prepare("SELECT id, owner_id, type FROM vertices ORDER BY id LIMIT ?")
                    }
                }
            },
            VertexQuery::Vertex(id) => {
                self.conn.prepare("SELECT id, owner_id, type FROM vertices WHERE id=? LIMIT 1")
            },
            VertexQuery::Vertices(vertices) => {
                let mut params_template_builder = vec![];

                for id in vertices {
                    params_template_builder.push("?");
                }

                let query_template = format!("SELECT id, owner_id, type FROM vertices WHERE id IN ({}) ORDER BY id", params_template_builder.join(", "));
                self.conn.prepare(query_template)
            },
            VertexQuery::Pipe(edge_query, converter, limit) => {
                self.edge_query_to_sql(*edge_query, sql_query_builder);
                let params: Vec<Box<ToSql>> = vec![Box::new(limit as i64)];

                let query_template = match converter {
                    QueryTypeConverter::Outbound => "SELECT id, owner_id, type FROM vertices WHERE id IN (SELECT outbound_id FROM %t) ORDER BY id LIMIT %p",
                    QueryTypeConverter::Inbound => "SELECT id, owner_id, type FROM vertices WHERE id IN (SELECT inbound_id FROM %t) ORDER BY id LIMIT %p"
                };

                sql_query_builder.push(query_template, "", params);
            }
        }
    }
    
    fn edge_query_to_sql(&self, q: EdgeQuery) -> String {
        match q {
            EdgeQuery::Edge(key) => {
                "SELECT id, outbound_id, type, inbound_id, update_timestamp, weight FROM edges WHERE outbound_id=? AND type=? AND inbound_id=?"
            },
            EdgeQuery::Edges(edges) => {
                let mut params_template_builder = vec![];

                for key in edges {
                    params_template_builder.push("(?, ?, ?)");
                }

                format!("SELECT id, outbound_id, type, inbound_id, update_timestamp, weight FROM edges WHERE (outbound_id, type, inbound_id) IN ({})", params_template_builder.join(", "));
            },
            EdgeQuery::Pipe(vertex_query, converter, t, high, low, limit) => {
                self.vertex_query_to_sql(*vertex_query, sql_query_builder);

                let mut where_clause_template_builder = vec![];

                if let Some(t) = t {
                    where_clause_template_builder.push("type = ?");
                }

                if let Some(high) = high {
                    where_clause_template_builder.push("update_timestamp <= ?");
                }

                if let Some(low) = low {
                    where_clause_template_builder.push("update_timestamp >= ?");
                }

                params.push(Box::new(limit as i64));
                let where_clause = where_clause_template_builder.join(" AND ");

                let query_template = match (converter, where_clause.len()) {
                    (QueryTypeConverter::Outbound, 0) => {
                        "SELECT id, outbound_id, type, inbound_id, update_timestamp, weight FROM edges WHERE outbound_id IN (SELECT id FROM %t) ORDER BY update_timestamp DESC LIMIT %p".to_string()
                    },
                    (QueryTypeConverter::Outbound, _) => {
                        format!("SELECT id, outbound_id, type, inbound_id, update_timestamp, weight FROM edges WHERE outbound_id IN (SELECT id FROM %t) AND {} ORDER BY update_timestamp DESC LIMIT %p", where_clause)
                    },
                    (QueryTypeConverter::Inbound, 0) => {
                        "SELECT id, outbound_id, type, inbound_id, update_timestamp, weight FROM edges WHERE inbound_id IN (SELECT id FROM %t) ORDER BY update_timestamp DESC LIMIT %p".to_string()
                    },
                    (QueryTypeConverter::Inbound, _) => {
                        format!("SELECT id, outbound_id, type, inbound_id, update_timestamp, weight FROM edges WHERE inbound_id IN (SELECT id FROM %t) AND {} ORDER BY update_timestamp DESC LIMIT %p", where_clause)
                    }
                 };
                
                sql_query_builder.push(&query_template[..], "", params);
            }
        }
    }
}

impl Transaction for SQLiteTransaction {
    fn commit(self) -> Result<(), Error> {
        self.conn.execute("COMMIT TRANSACTION", &[])?;
        Ok(())
    }

    fn rollback(self) -> Result<(), Error> {
        self.conn.execute("ROLLBACK TRANSACTION", &[])?;
        Ok(())
    }

    fn create_vertex(&self, t: models::Type) -> Result<Uuid, Error> {
        let id = if self.secure_uuids {
            parent_uuid()
        } else {
            child_uuid(self.account_id)
        };

        self.trans.execute("INSERT INTO vertices (id, type, owner_id) VALUES (?, ?, ?)", &[&id, &t.0, &self.account_id])?;
        Ok(id)
    }

    fn get_vertices(&self, q: models::VertexQuery) -> Result<Vec<models::Vertex>, Error> {
        let statement: Statement = self.prepare_vertex_query(q);

        let mut vertices = Vec::new();

        let results = statement.query_map(&self.prepare_vertex_params(q), |row| {
            models::Vertex::new(Uuid::from_str(row.get(0)), Type::new(row.get(1))        
        });

        for result in results {
            vertices.push(result);
        }

        Ok(vertices)
    }
}