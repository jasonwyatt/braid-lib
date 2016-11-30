use datastore::{Datastore, Transaction};
use super::sandbox::DatastoreTestSandbox;
use errors::Error;
use models;
use traits::Id;
use serde_json::Value as JsonValue;

pub fn should_handle_global_metadata<D: Datastore<T, I>, T: Transaction<I>, I: Id>(sandbox: &mut DatastoreTestSandbox<D, T, I>) {
	let key = sandbox.generate_unique_string("global-metadata");
	let trans = sandbox.transaction();

	// Check to make sure there's no initial value
	let result = trans.get_global_metadata(key.clone());
	assert_eq!(result.unwrap_err(), Error::MetadataNotFound);

	// Set and get the value as true
	let result = trans.set_global_metadata(key.clone(), JsonValue::Bool(true));
	assert!(result.is_ok());

	let result = trans.get_global_metadata(key.clone());
	assert_eq!(result.unwrap(), JsonValue::Bool(true));

	// Set and get the value as false
	let result = trans.set_global_metadata(key.clone(), JsonValue::Bool(false));
	assert!(result.is_ok());

	let result = trans.get_global_metadata(key.clone());
	assert_eq!(result.unwrap(), JsonValue::Bool(false));

	// Delete & check that it's deleted
	let result = trans.delete_global_metadata(key.clone());
	assert!(result.is_ok());

	let result = trans.get_global_metadata(key.clone());
	assert_eq!(result.unwrap_err(), Error::MetadataNotFound);
}

pub fn should_handle_account_metadata<D: Datastore<T, I>, T: Transaction<I>, I: Id>(sandbox: &mut DatastoreTestSandbox<D, T, I>) {
	let key = sandbox.generate_unique_string("account-metadata");
	let trans = sandbox.transaction();

	// Check to make sure there's no initial value
	let result = trans.get_account_metadata(sandbox.owner_id, key.clone());
	assert_eq!(result.unwrap_err(), Error::MetadataNotFound);

	// Set and get the value as true
	let result = trans.set_account_metadata(sandbox.owner_id, key.clone(), JsonValue::Bool(true));
	assert!(result.is_ok());

	let result = trans.get_account_metadata(sandbox.owner_id, key.clone());
	assert_eq!(result.unwrap(), JsonValue::Bool(true));

	// Set and get the value as false
	let result = trans.set_account_metadata(sandbox.owner_id, key.clone(), JsonValue::Bool(false));
	assert!(result.is_ok());

	let result = trans.get_account_metadata(sandbox.owner_id, key.clone());
	assert_eq!(result.unwrap(), JsonValue::Bool(false));

	// Delete & check that it's deleted
	let result = trans.delete_account_metadata(sandbox.owner_id, key.clone());
	assert!(result.is_ok());

	let result = trans.get_account_metadata(sandbox.owner_id, key.clone());
	assert_eq!(result.unwrap_err(), Error::MetadataNotFound);
}

pub fn should_handle_vertex_metadata<D: Datastore<T, I>, T: Transaction<I>, I: Id>(sandbox: &mut DatastoreTestSandbox<D, T, I>) {
	let trans = sandbox.transaction();
	let t = models::Type::new("test_edge_type".to_string()).unwrap();
	let owner_id = trans.create_vertex(t).unwrap();
	let key = sandbox.generate_unique_string("vertex-metadata");

	// Check to make sure there's no initial value
	let result = trans.get_vertex_metadata(owner_id, key.clone());
	assert_eq!(result.unwrap_err(), Error::MetadataNotFound);

	// Set and get the value as true
	let result = trans.set_vertex_metadata(owner_id, key.clone(), JsonValue::Bool(true));
	assert!(result.is_ok());

	let result = trans.get_vertex_metadata(owner_id, key.clone());
	assert_eq!(result.unwrap(), JsonValue::Bool(true));

	// Set and get the value as false
	let result = trans.set_vertex_metadata(owner_id, key.clone(), JsonValue::Bool(false));
	assert!(result.is_ok());

	let result = trans.get_vertex_metadata(owner_id, key.clone());
	assert_eq!(result.unwrap(), JsonValue::Bool(false));

	// Delete & check that it's deleted
	let result = trans.delete_vertex_metadata(owner_id, key.clone());
	assert!(result.is_ok());

	let result = trans.get_vertex_metadata(owner_id, key.clone());
	assert_eq!(result.unwrap_err(), Error::MetadataNotFound);
}

pub fn should_handle_edge_metadata<D: Datastore<T, I>, T: Transaction<I>, I: Id>(sandbox: &mut DatastoreTestSandbox<D, T, I>) {
	let trans = sandbox.transaction();
	let vertex_t = models::Type::new("test_edge_type".to_string()).unwrap();
	let outbound_id = trans.create_vertex(vertex_t.clone()).unwrap();
	let inbound_id = trans.create_vertex(vertex_t).unwrap();
	let edge_t = models::Type::new("test_edge_type".to_string()).unwrap();
	let e = models::Edge::new(outbound_id, edge_t.clone(), inbound_id, models::Weight::new(0.5).unwrap());
	trans.set_edge(e).unwrap();

	let key = sandbox.generate_unique_string("edge-metadata");

	// Check to make sure there's no initial value
	let result = trans.get_edge_metadata(outbound_id, edge_t.clone(), inbound_id, key.clone());
	assert_eq!(result.unwrap_err(), Error::MetadataNotFound);

	// Set and get the value as true
	let result = trans.set_edge_metadata(outbound_id, edge_t.clone(), inbound_id, key.clone(), JsonValue::Bool(true));
	assert!(result.is_ok());

	let result = trans.get_edge_metadata(outbound_id, edge_t.clone(), inbound_id, key.clone());
	assert_eq!(result.unwrap(), JsonValue::Bool(true));

	// Set and get the value as false
	let result = trans.set_edge_metadata(outbound_id, edge_t.clone(), inbound_id, key.clone(), JsonValue::Bool(false));
	assert!(result.is_ok());

	let result = trans.get_edge_metadata(outbound_id, edge_t.clone(), inbound_id, key.clone());
	assert_eq!(result.unwrap(), JsonValue::Bool(false));

	// Delete & check that it's deleted
	let result = trans.delete_edge_metadata(outbound_id, edge_t.clone(), inbound_id, key.clone());
	assert!(result.is_ok());

	let result = trans.get_edge_metadata(outbound_id, edge_t, inbound_id, key.clone());
	assert_eq!(result.unwrap_err(), Error::MetadataNotFound);
}