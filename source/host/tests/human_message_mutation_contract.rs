use std::fs;

fn read_source(relative: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(relative)).expect("contract source must be readable")
}

#[test]
fn reg_msg_mutation_transport_001_uses_canonical_patch_delete_and_revision_fencing() {
    let source = read_source("src/extensions/session/native_messaging.rs");
    assert!(source.contains("pub fn edit_direct_message("));
    assert!(source.contains(".patch(self.endpoint(&path)?)"));
    assert!(source.contains("pub fn delete_direct_message("));
    assert!(source.contains(".delete(self.endpoint(&path)?)"));
    assert!(source.contains("expected_revision"));
    assert!(source.contains("operation_id"));
    assert!(source.contains("mismatched message id"));
    assert!(source.contains("non-advancing"));
}

#[test]
fn reg_msg_mutation_settlement_001_reuses_the_existing_transcript_owner() {
    let source = read_source("src/extensions/session/production.rs");
    assert!(source.contains("pub fn edit_human_message("));
    assert!(source.contains("pub fn delete_human_message("));
    assert!(source.contains("self.materialize_remote_human_message(&owner, &remote)"));
    assert!(source.contains("\"remoteRevision\": remote.revision.unwrap_or(1).max(1)"));
    assert!(source.contains("\"isDeleted\": remote.is_deleted"));
    assert!(source.contains("only the local sender may edit a Human message"));
    assert!(source.contains("only the local sender may delete a Human message"));
}

#[test]
fn reg_msg_mutation_gateway_001_exposes_typed_human_commands() {
    let source = read_source("src/extensions/session/gateway.rs");
    assert!(source.contains("\"editHumanMessage\""));
    assert!(source.contains("\"deleteHumanMessage\""));
    assert!(source.contains("required_string(args, \"operationId\")"));
}
