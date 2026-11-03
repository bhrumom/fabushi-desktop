//! MSU-v1 causal App controls. All ownership, SQL and generated Runner paths
//! are production paths. COMMIT/ROLLBACK uncertainty uses only the exact,
//! one-use SandAgentDb command-edge leases. INSERT triggers supply ordinary
//! refusal controls; local gates below only control scheduling.

use super::*;
use super::generated_send_message_fidelity::{
    Field, OwnedChild, Responses, fields, file_bytes, optional_bytes,
};
use super::super::UnusedCompactionProviderPorts;
use mahayana_host_runtime::attachment_paths::get_agent_media_store_roots;
use mahayana_host_runtime::extensions::inference::provider_session::{ProviderMessage, RoutedProvider};
use mahayana_host_runtime::extensions::session::agent_db::{
    MediaSettlementTestFault, SessionWorkAdmissionRecord,
};
use mahayana_host_runtime::extensions::session::gateway::persist_accepted_send_prompt_context;
use mahayana_host_runtime::extensions::transcript::production_runtime::ProductionSendError;
use mahayana_host_runtime::extensions::transcript::sand_ack_obligation_store::SandAckObligationStore;
use mahayana_host_runtime::runner::production_agent_checkpoint::{
    AgentStateCheckpointSink, reconcile_completed_original_turn,
};
use mahayana_host_runtime::runner::production_turn_agent_owner::{
    ProductionTurnAgentBuildBindings, ProductionTurnAgentLifecycleBindings,
    ProductionTurnAgentStaticConfig,
};
use mahayana_host_runtime::runner::routed_provider_runtime::{
    ProductionRoutedProviderCheckpointStore, RunnerRequestContextSnapshot,
};
use mahayana_host_runtime::runner::tools::send_message_tool::{
    SendMessageToolBridge, send_message_tool_definition,
};
use mahayana_host_runtime::runner::turn_agent_composition::{SAND_AGENT_TOKEN_LIMIT, TurnAgentComposition};
use mahayana_host_runtime::runner::turn_observation::TurnObservation;
use mahayana_host_runtime::runner::{
    TerminalOutcome, ToolEffectCustodyObservation, ToolEffectCustodyReport,
    ToolEffectCustodySnapshot, ToolEffectUnknownResult, TurnRunOptions,
};
use mahayana_host_runtime::runner_production_bridge::create_production_runner;
use rusqlite::{Connection, OpenFlags};
use std::collections::BTreeMap;
use std::io::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const SOURCE_BYTES: &[u8] = b"MSU original bytes\0\xff\n";
const SIBLING_BYTES: &[u8] = b"unrelated managed sibling must survive\n";
const SOURCE_NAME: &str = "media source 雪.bin";
const DIRECT_ID: &str = "msu-direct-media";
const GENERATED_ID: &str = "msu-generated-media";
const STREAM: &str = "msu-generated-stream";
const MODEL: &str = "Vendor/Media/Settlement@ActualApp";
const PROMPT: &str = "Send the local attachment exactly once.";
const CHILD_REQUEST_ENV: &str = "FABUSHI_MEDIA_SETTLEMENT_REQUEST";
const GENERATED_TEST: &str = "tests::channel_host_lane_tests::media_settlement_unknown::media_settlement_app_generated_runner_lost_ack_retains_unknown_custody";

type RawRow = (i64, String, Vec<u8>);

#[derive(Debug, Clone, PartialEq)]
struct DurableImage {
    rows: Vec<RawRow>,
    admission: SessionWorkAdmissionRecord,
}

fn reader(path: &Path) -> Connection {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("independent file-backed read, without owner recovery")
}

fn durable_image(path: &Path) -> DurableImage {
    let db = reader(path);
    let mut statement = db.prepare("SELECT seq, id, CAST(entry AS BLOB) FROM transcript_entries ORDER BY seq").unwrap();
    let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap().map(Result::unwrap).collect();
    let admission: String = db.query_row(
        "SELECT value FROM kv WHERE key = 'sessionWorkAdmission'", [], |row| row.get(0),
    ).unwrap();
    DurableImage { rows, admission: serde_json::from_str(&admission).unwrap() }
}

fn values(rows: &[RawRow]) -> Vec<Value> {
    rows.iter().map(|(_, _, bytes)| serde_json::from_slice(bytes).unwrap()).collect()
}

fn released(mut admission: SessionWorkAdmissionRecord) -> SessionWorkAdmissionRecord {
    admission.admitted_media_work_id = None;
    admission.admitted_media_paths.clear();
    admission
}

fn media_bytes(fixture: &ChannelHostLaneFixture) -> BTreeMap<PathBuf, Vec<u8>> {
    get_agent_media_store_roots(fixture.root.join("agents").join(&fixture.agent_id))
        .into_iter().flat_map(|root| {
            file_bytes(&root).into_iter().map(move |(path, bytes)| (root.join(path), bytes))
        }).collect()
}

struct AckEvidence {
    token: String,
    obligation: Value,
    path: PathBuf,
    bytes: Vec<u8>,
}

impl AckEvidence {
    fn assert_retained(&self, root: &Path, agent_id: &str) {
        assert_eq!(fs::read(&self.path).unwrap(), self.bytes, "no ACK fulfillment or rewrite");
        assert_eq!(serde_json::to_value(SandAckObligationStore::new(root)
            .get_checked(agent_id).unwrap().expect("real outstanding ACK obligation")).unwrap(), self.obligation);
    }

    fn assert_bound(&self, sink: &ProductionSendMessageSink) {
        assert_eq!(sink.ack_token.as_deref(), Some(self.token.as_str()));
        assert!(sink.ack_obligations.is_ack_run_token_for_agent(&sink.agent_id, &self.token).unwrap());
    }
}

fn take_sink_with_ack(fixture: &mut ChannelHostLaneFixture) -> (Arc<ProductionSendMessageSink>, AckEvidence) {
    let mut sink = fixture.sink.take().expect("the fixture's actual private App sink");
    let reservation = sink.ack_obligations.record_send_and_mint_token(&fixture.agent_id, 1_234.0).unwrap();
    assert!(reservation.created);
    sink.ack_token = Some(reservation.ack_token.clone());
    let path = sink.ack_obligations.store().file_path().to_path_buf();
    let ack = AckEvidence {
        token: reservation.ack_token,
        obligation: serde_json::to_value(reservation.obligation).unwrap(),
        bytes: fs::read(&path).unwrap(), path,
    };
    ack.assert_bound(&sink);
    ack.assert_retained(&fixture.root, &fixture.agent_id);
    (Arc::new(sink), ack)
}

#[derive(Clone)]
struct PreparedMedia {
    source: PathBuf,
    source_url: String,
    resolved: ResolvedAttachmentSource,
    owned: PathBuf,
    sibling: PathBuf,
}

impl PreparedMedia {
    fn materialize(fixture: &ChannelHostLaneFixture, sink: &ProductionSendMessageSink, id: &str) -> Self {
        let source = fixture.root.join(SOURCE_NAME);
        fs::write(&source, SOURCE_BYTES).unwrap();
        let source_url = file_url_for_path(&source).unwrap();
        let resolved = sink.resolve_attachment_source(&source_url, id).expect("real local materialization");
        let owned = file_path_from_file_url(&resolved.url).unwrap();
        assert_ne!(source, owned);
        assert_eq!(resolved.file_name.as_deref(), Some(SOURCE_NAME));
        assert!(owned.starts_with(fixture.root.join("agents").join(&fixture.agent_id)));
        let sibling = owned.parent().unwrap().join("unrelated-msu-sibling.bin");
        fs::write(&sibling, SIBLING_BYTES).unwrap();
        let media = Self { source, source_url, resolved, owned, sibling };
        media.assert_bytes(true);
        let admission = durable_image(&fixture.deps().session_workers.session_db_path(&fixture.agent_id).unwrap()).admission;
        if sink.session_key.is_some() {
            assert_eq!(admission.admitted_media_work_id.as_deref(), Some(format!("send-media:{id}").as_str()));
            assert_eq!(admission.admitted_media_paths, vec![media.owned.to_str().unwrap().to_string()]);
        } else {
            assert!(admission.admitted_media_work_id.is_none());
            assert!(admission.admitted_media_paths.is_empty());
        }
        media
    }

    fn message(&self, channel: bool) -> Value {
        let mut message = json!({"type":"attachment", "url":self.resolved.url,
            "file_name":self.resolved.file_name, "alt":"retained MSU attachment"});
        if channel { message["channel"] = json!(ADDRESS); }
        message
    }

    fn assert_bytes(&self, retained: bool) {
        assert_eq!(fs::read(&self.source).unwrap(), SOURCE_BYTES);
        assert_eq!(fs::read(&self.sibling).unwrap(), SIBLING_BYTES);
        if retained { assert_eq!(fs::read(&self.owned).unwrap(), SOURCE_BYTES); }
        else { assert!(!self.owned.exists(), "only the captured uncommitted original is removed"); }
    }
}

fn assert_unknown<T: std::fmt::Debug>(result: Result<T, ProviderSessionError>) {
    assert!(matches!(result, Err(ProviderSessionError::ToolOutcomeUnknown(_))),
        "the actual sink must preserve typed uncertainty: {result:?}");
}

fn pending_entry(sink: &ProductionSendMessageSink, id: &str) -> Arc<Mutex<PendingOutgoingMedia>> {
    sink.pending_media.lock().unwrap().get(id).cloned().expect("retained exact media identity")
}

fn assert_unknown_entry(sink: &ProductionSendMessageSink, id: &str, admission: &SessionWorkAdmissionRecord, media: &PreparedMedia) {
    let entry = pending_entry(sink, id);
    let pending = entry.try_lock().expect("the returned unknown is not still actively executing");
    assert_eq!(pending.phase, PendingOutgoingMediaPhase::OwnerSettlementPending);
    assert!(pending.custody.is_owner_settlement_pending());
    assert!(pending.has_owned_media);
    assert_eq!(pending.session_key.as_deref(), Some(admission.session_key.as_str()));
    assert_eq!(pending.lifecycle_revision.as_deref(), Some(admission.lifecycle_revision.as_str()));
    assert_eq!(pending.media_work_id.as_deref(), Some(format!("send-media:{id}").as_str()));
    assert_eq!(pending.resolved_sources.len(), 1);
    assert_eq!(pending.resolved_sources.get(&media.source_url), Some(&media.resolved));
}

fn assert_terminal(sink: &ProductionSendMessageSink, id: &str, phase: PendingOutgoingMediaPhase) {
    let entry = pending_entry(sink, id);
    let pending = entry.try_lock().unwrap();
    assert_eq!(pending.phase, phase);
    assert!(!pending.custody.is_owner_settlement_pending());
    assert!(!pending.has_owned_media);
    assert!(pending.media_work_id.is_none());
    assert!(pending.resolved_sources.is_empty());
}

fn assert_quiet(fixture: &ChannelHostLaneFixture, port: &ControlledChannelPort) {
    assert!(port.calls().is_empty(), "unknown/refused sends cannot enter the real transport");
    assert!(port.effects().is_empty());
    assert!(matches!(fixture.lane_observations.try_recv(), Err(mpsc::TryRecvError::Empty)),
        "no deliverToChannel request may be queued");
    fixture.assert_no_worker_observation();
    fixture.assert_no_failure();
}

fn assert_committed_image(before: &DurableImage, after: &DurableImage, message: &Value, id: &str) {
    assert_eq!(&after.rows[..before.rows.len()], before.rows.as_slice());
    assert_eq!(after.rows.len(), before.rows.len() + 2, "one send and one hidden settlement receipt");
    let entries = values(&after.rows[before.rows.len()..]);
    let send = &entries[0];
    assert_eq!(send["kind"], "send-message");
    assert_eq!(send["sessionKey"], before.admission.session_key);
    assert_eq!(&send["message"], message);
    assert!(send["id"].as_str().is_some_and(|id| !id.is_empty()));
    assert_eq!(entries[1], json!({
        "id":format!("outgoing-media-settlement:send-media:{id}"),
        "kind":"outgoing-media-settlement", "sessionKey":before.admission.session_key,
        "mediaWorkId":format!("send-media:{id}"), "lifecycleRevision":before.admission.lifecycle_revision,
        "mediaPaths":before.admission.admitted_media_paths, "transcriptEntryIds":[send["id"]],
    }));
    assert_eq!(after.admission, released(before.admission.clone()));
}

fn refuse_receipt_after_real_prefix(path: &Path) {
    // This trigger can refuse only after the very send named by the incoming
    // receipt exists in this transaction. Without that prefix its SELECT is
    // empty; RefuseRollback cannot be applied by this fixture's command path.
    Connection::open(path).unwrap().execute_batch(
        "CREATE TRIGGER msu_app_refuse_receipt BEFORE INSERT ON transcript_entries
         WHEN json_extract(NEW.entry, '$.kind') = 'outgoing-media-settlement'
         BEGIN SELECT RAISE(ABORT, 'MSU receipt refused after actual send INSERT')
           WHERE EXISTS (
             SELECT 1 FROM transcript_entries AS inserted
             WHERE inserted.id = json_extract(NEW.entry, '$.transcriptEntryIds[0]')
               AND json_extract(inserted.entry, '$.kind') = 'send-message'
               AND json_extract(inserted.entry, '$.sessionKey') = json_extract(NEW.entry, '$.sessionKey')
           ); END;",
    ).unwrap();
}

fn close_for_reopen(fixture: ChannelHostLaneFixture) -> (PathBuf, String) {
    let sessions = Arc::clone(&fixture.deps().session_workers);
    let identity = fixture.stop_for_reopen();
    sessions.shutdown_with_checkpoint();
    assert_eq!(sessions.active_worker_count(), 0);
    drop(sessions);
    identity
}

fn assert_two_fresh_owners(
    root: &Path, agent_id: &str, db_path: &Path, expected: &DurableImage,
    media: &PreparedMedia, retained: bool, ack: &AckEvidence,
) {
    for reopen in 0..2 {
        let fresh = ChannelHostLaneFixture::open("msu-fresh-owner", root.to_path_buf(), Some(agent_id.to_string()));
        assert_eq!(durable_image(db_path), *expected, "fresh owner {reopen}: exact rows/receipt/fence");
        media.assert_bytes(retained);
        ack.assert_retained(root, agent_id);
        let visible = values(&expected.rows).into_iter()
            .filter(|entry| entry["kind"] != "outgoing-media-settlement").collect::<Vec<_>>();
        assert_eq!(fresh.entries(), visible, "the hidden receipt cannot become a visible message");
        fresh.assert_no_followup(&visible);
        fresh.assert_no_failure();
        assert!(matches!(fresh.lane_observations.try_recv(), Err(mpsc::TryRecvError::Empty)));
        close_for_reopen(fresh);
        assert_eq!(durable_image(db_path), *expected);
        media.assert_bytes(retained);
    }
}

fn assert_fresh_owner_can_settle_new_media(root: &Path, agent: &str, ack: &AckEvidence) {
    let mut fresh = ChannelHostLaneFixture::open("msu-recovered-positive", root.to_path_buf(), Some(agent.to_string()));
    // A separate new invocation, after both first-open recovery observations.
    // Its normal fixture sink has no token for the earlier independent ACK.
    let sink = Arc::new(fresh.sink.take().unwrap());
    assert!(sink.ack_token.is_none());
    let id = "msu-new-work-after-safe-recovery";
    let media = PreparedMedia::materialize(&fresh, &sink, id);
    let path = fresh.deps().session_workers.session_db_path(agent).unwrap();
    let before = durable_image(&path);
    let counter = SendMessageDeliveryCounter::default();
    let counted = CountingSendMessageSink::new(sink.clone(), counter.clone());
    assert!(counted.send_message(media.message(false), 8_000, id).unwrap().is_some());
    assert_eq!(counter.count(), 1, "ordinary positive settlement remains available to the fresh owner");
    assert_committed_image(&before, &durable_image(&path), &media.message(false), id);
    assert_terminal(&sink, id, PendingOutgoingMediaPhase::Committed);
    media.assert_bytes(true);
    ack.assert_retained(root, agent);
    drop((counted, sink));
    close_for_reopen(fresh);
}

#[test]
fn media_settlement_app_known_receipt_refusal_rolls_back_only_owned_media() {
    let mut fixture = ChannelHostLaneFixture::new("msu-known-refusal");
    let (port, _) = fixture.register_transport(TransportOutcome::Delivered, false);
    let (sink, ack) = take_sink_with_ack(&mut fixture);
    let media = PreparedMedia::materialize(&fixture, &sink, DIRECT_ID);
    let owner = fixture.deps().session_workers.open_agent_db_owner(&fixture.agent_id).unwrap();
    let path = owner.db_path().to_path_buf();
    let before = durable_image(&path);
    refuse_receipt_after_real_prefix(&path);
    let events = fixture.deps().events.subscribe();
    let counter = SendMessageDeliveryCounter::default();
    let counted = CountingSendMessageSink::new(sink.clone(), counter.clone());
    let error = counted.send_message(media.message(true), 2_000, DIRECT_ID).unwrap_err();
    let ProviderSessionError::Tool(detail) = error else { panic!("confirmed rollback is an ordinary refusal: {error:?}"); };
    assert!(detail.contains("MSU receipt refused after actual send INSERT"), "real prefix witness: {detail}");
    assert!(!owner.is_closed(), "confirmed rollback leaves this owner usable");
    let expected = DurableImage { rows: before.rows.clone(), admission: released(before.admission.clone()) };
    assert_eq!(durable_image(&path), expected);
    media.assert_bytes(false);
    assert_terminal(&sink, DIRECT_ID, PendingOutgoingMediaPhase::Aborted);
    let old = pending_entry(&sink, DIRECT_ID);
    assert!(matches!(sink.resolve_attachment_source(&media.source_url, DIRECT_ID), Err(ProviderSessionError::Tool(_))));
    assert!(matches!(counted.send_message(media.message(true), 2_001, DIRECT_ID), Err(ProviderSessionError::Tool(_))));
    sink.abort_attachment_custody(DIRECT_ID);
    assert!(Arc::ptr_eq(&old, &pending_entry(&sink, DIRECT_ID)));
    assert_eq!(durable_image(&path), expected);
    assert_eq!(counter.count(), 0);
    ack.assert_bound(&sink);
    ack.assert_retained(&fixture.root, &fixture.agent_id);
    assert!(matches!(events.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_quiet(&fixture, &port);
    drop((old, counted));
    let weak = Arc::downgrade(&sink);
    drop(sink);
    assert!(weak.upgrade().is_none(), "no retained observer Arc may mask the real sink destructor");
    media.assert_bytes(false);
    drop(owner);
    let (root, agent) = close_for_reopen(fixture);
    assert_two_fresh_owners(&root, &agent, &path, &expected, &media, false, &ack);
    fs::remove_dir_all(root).unwrap();
}

fn direct_unknown(fault_kind: MediaSettlementTestFault) {
    let committed = fault_kind == MediaSettlementTestFault::LoseCommitAcknowledgement;
    let mut fixture = ChannelHostLaneFixture::new(if committed { "msu-lost-ack" } else { "msu-denied-rollback" });
    let (port, _) = fixture.register_transport(TransportOutcome::Delivered, false);
    let (sink, ack) = take_sink_with_ack(&mut fixture);
    let media = PreparedMedia::materialize(&fixture, &sink, DIRECT_ID);
    let owner = fixture.deps().session_workers.open_agent_db_owner(&fixture.agent_id).unwrap();
    let path = owner.db_path().to_path_buf();
    let before = durable_image(&path);
    if !committed { refuse_receipt_after_real_prefix(&path); }
    let fault = owner.arm_media_settlement_fault_for_test(
        &before.admission.session_key, &before.admission.lifecycle_revision,
        &format!("send-media:{DIRECT_ID}"), fault_kind,
    ).unwrap();
    assert!(!fault.was_claimed() && !fault.was_applied());
    let events = fixture.deps().events.subscribe();
    let counter = SendMessageDeliveryCounter::default();
    let counted = CountingSendMessageSink::new(sink.clone(), counter.clone());
    let message = media.message(true);
    assert_unknown(counted.send_message(message.clone(), 3_000, DIRECT_ID));
    assert!(fault.was_claimed());
    assert!(fault.was_applied(), "the real SQL command edge must be reached, not a fabricated unknown return");
    assert!(owner.is_closed(), "the exact uncertain connection is retired before the App returns");
    let after = durable_image(&path);
    if committed { assert_committed_image(&before, &after, &message, DIRECT_ID); }
    else { assert_eq!(after, before, "closing the denied-rollback connection leaves no committed send/receipt"); }
    assert_unknown_entry(&sink, DIRECT_ID, &before.admission, &media);
    let old = pending_entry(&sink, DIRECT_ID);
    let files = media_bytes(&fixture);
    // An invalid changed URL proves this refusal precedes URL conversion/I/O.
    assert_unknown(sink.resolve_attachment_source(&format!("{}%00", media.source_url), DIRECT_ID));
    assert_unknown(counted.send_message(text_message("same ID must not allocate another history entry"), 3_001, DIRECT_ID));
    sink.abort_attachment_custody(DIRECT_ID);
    assert!(Arc::ptr_eq(&old, &pending_entry(&sink, DIRECT_ID)));
    assert_unknown_entry(&sink, DIRECT_ID, &before.admission, &media);
    assert_eq!(durable_image(&path), after);
    assert_eq!(media_bytes(&fixture), files);
    // The old Session's read path may refuse its retired owner. It cannot
    // promote a cached read into first-open media cleanup or owner replacement.
    let _ = fixture.deps().session_workers.read_session_work_admission(&fixture.agent_id);
    assert!(owner.is_closed());
    assert_eq!(durable_image(&path), after);
    media.assert_bytes(true);
    assert_eq!(counter.count(), 0);
    ack.assert_bound(&sink);
    ack.assert_retained(&fixture.root, &fixture.agent_id);
    assert!(matches!(events.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_quiet(&fixture, &port);
    drop((old, counted, fault));
    let weak = Arc::downgrade(&sink);
    drop(sink);
    assert!(weak.upgrade().is_none());
    media.assert_bytes(true);
    assert_eq!(durable_image(&path), after, "last App Arc Drop cannot release the durable fence");
    drop(owner);
    let (root, agent) = close_for_reopen(fixture);
    assert_eq!(durable_image(&path), after, "disposal alone is not recovery");
    media.assert_bytes(true);
    let expected = DurableImage { rows: after.rows.clone(), admission: released(after.admission.clone()) };
    assert_two_fresh_owners(&root, &agent, &path, &expected, &media, committed, &ack);
    if !committed {
        Connection::open(&path).unwrap().execute_batch("DROP TRIGGER msu_app_refuse_receipt").unwrap();
    }
    assert_fresh_owner_can_settle_new_media(&root, &agent, &ack);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn media_settlement_app_lost_commit_ack_retains_rows_bytes_and_same_id_fence() {
    direct_unknown(MediaSettlementTestFault::LoseCommitAcknowledgement);
}

#[test]
fn media_settlement_app_denied_rollback_retains_until_first_open_recovery() {
    direct_unknown(MediaSettlementTestFault::RefuseRollback);
}

#[test]
fn media_settlement_app_legacy_owned_refusal_retains_without_inventing_receipt() {
    let mut fixture = ChannelHostLaneFixture::new("msu-legacy-owned");
    fixture.sink.as_mut().unwrap().session_key = None;
    let (port, _) = fixture.register_transport(TransportOutcome::Delivered, false);
    let (sink, ack) = take_sink_with_ack(&mut fixture);
    let media = PreparedMedia::materialize(&fixture, &sink, DIRECT_ID);
    let owner = fixture.deps().session_workers.open_agent_db_owner(&fixture.agent_id).unwrap();
    let path = owner.db_path().to_path_buf();
    let before = durable_image(&path);
    assert!(before.admission.admitted_media_work_id.is_none());
    Connection::open(&path).unwrap().execute_batch(
        "CREATE TRIGGER msu_legacy_refuse_send BEFORE INSERT ON transcript_entries
         WHEN json_extract(NEW.entry, '$.kind') = 'send-message'
         BEGIN SELECT RAISE(ABORT, 'MSU actual legacy INSERT refusal'); END;",
    ).unwrap();
    let publications = fixture.deps().events.subscribe();
    let counter = SendMessageDeliveryCounter::default();
    let counted = CountingSendMessageSink::new(sink.clone(), counter.clone());
    let text_error = counted.send_message(text_message("legacy INSERT control"), 3_499,
        "msu-legacy-untracked-text").unwrap_err();
    let ProviderSessionError::Tool(detail) = text_error else {
        panic!("the untracked text API retains ordinary refusal: {text_error:?}");
    };
    assert!(detail.contains("MSU actual legacy INSERT refusal"), "real legacy SQL control: {detail}");
    assert_unknown(counted.send_message(media.message(true), 3_500, DIRECT_ID));
    // This is the legacy String boundary's conservative retention fallback,
    // not a claimed COMMIT/ROLLBACK uncertainty injection or a new receipt.
    assert_eq!(durable_image(&path), before);
    assert!(!owner.is_closed(), "ordinary legacy refusal did not use either MSU command fault");
    {
        let entry = pending_entry(&sink, DIRECT_ID);
        let pending = entry.try_lock().unwrap();
        assert_eq!(pending.phase, PendingOutgoingMediaPhase::OwnerSettlementPending);
        assert!(pending.custody.is_owner_settlement_pending() && pending.has_owned_media);
        assert!(pending.session_key.is_none() && pending.lifecycle_revision.is_none() && pending.media_work_id.is_none());
        assert_eq!(pending.resolved_sources.get(&media.source_url), Some(&media.resolved));
    }
    assert_unknown(sink.resolve_attachment_source(&format!("{}%00", media.source_url), DIRECT_ID));
    assert_unknown(counted.send_message(media.message(true), 3_501, DIRECT_ID));
    sink.abort_attachment_custody(DIRECT_ID);
    assert_eq!(counter.count(), 0);
    assert_eq!(durable_image(&path), before);
    media.assert_bytes(true);
    ack.assert_retained(&fixture.root, &fixture.agent_id);
    assert!(matches!(publications.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_quiet(&fixture, &port);
    drop((counted, owner));
    let weak = Arc::downgrade(&sink);
    drop(sink);
    assert!(weak.upgrade().is_none());
    media.assert_bytes(true);
    let (root, agent) = close_for_reopen(fixture);
    assert_two_fresh_owners(&root, &agent, &path, &before, &media, true, &ack);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn media_settlement_app_collecting_cache_multi_image_and_terminal_identity() {
    const PNG: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
        0, 0, 0, 1, 0, 0, 0, 1, 8, 4, 0, 0, 0, 181, 28, 12, 2,
        0, 0, 0, 11, 73, 68, 65, 84, 120, 218, 99, 100, 248, 15, 0,
        1, 5, 1, 1, 39, 24, 227, 102, 0, 0, 0, 0, 73, 69, 78, 68,
        174, 66, 96, 130,
    ];
    let mut fixture = ChannelHostLaneFixture::new("msu-cache-images");
    let (port, _) = fixture.register_transport(TransportOutcome::Delivered, false);
    let (sink, ack) = take_sink_with_ack(&mut fixture);
    let first = fixture.root.join("first image 雪.png");
    let second = fixture.root.join("second image.png");
    fs::write(&first, PNG).unwrap();
    fs::write(&second, PNG).unwrap();
    let first_url = file_url_for_path(&first).unwrap();
    let second_url = file_url_for_path(&second).unwrap();
    let id = "msu-three-image-arguments";
    let resolved_first = sink.resolve_attachment_source(&first_url, id).unwrap();
    let path = fixture.deps().session_workers.session_db_path(&fixture.agent_id).unwrap();
    let one = durable_image(&path);
    let first_files = media_bytes(&fixture);
    assert_eq!(first_files.len(), 1);
    // Deliberately change only the test's source between resolutions: exact
    // cache reuse must return the first owned bytes without rereading it.
    fs::write(&first, b"changed source must not be rematerialized").unwrap();
    assert_eq!(sink.resolve_attachment_source(&first_url, id).unwrap(), resolved_first);
    assert_eq!(durable_image(&path), one);
    assert_eq!(media_bytes(&fixture), first_files);
    fs::write(&first, PNG).unwrap();
    let resolved_second = sink.resolve_attachment_source(&second_url, id).unwrap();
    assert_ne!(resolved_first.url, resolved_second.url);
    let before = durable_image(&path);
    let first_owned = file_path_from_file_url(&resolved_first.url).unwrap();
    let second_owned = file_path_from_file_url(&resolved_second.url).unwrap();
    assert_eq!(before.admission.admitted_media_paths,
        vec![first_owned.to_str().unwrap().to_string(), second_owned.to_str().unwrap().to_string()]);
    assert_eq!(media_bytes(&fixture).len(), 2);
    let old = pending_entry(&sink, id);
    {
        let collecting = old.try_lock().unwrap();
        assert_eq!(collecting.phase, PendingOutgoingMediaPhase::Collecting);
        assert_eq!(collecting.resolved_sources.len(), 2);
        assert_eq!(collecting.resolved_sources.get(&first_url), Some(&resolved_first));
        assert_eq!(collecting.resolved_sources.get(&second_url), Some(&resolved_second));
    }
    let counter = SendMessageDeliveryCounter::default();
    let counted: Arc<dyn SendMessageSink> = Arc::new(CountingSendMessageSink::new(sink.clone(), counter.clone()));
    let bridge = SendMessageToolBridge::new(Arc::new(UnusedCompactionProviderPorts), counted.clone());
    let args = json!({"type":"text", "content":"one batch, two unique sources",
        "images":[{"url":first_url,"alt":"first"}, {"url":first_url,"alt":"repeat"},
            {"url":second_url,"alt":"second"}]});
    let result = bridge.call_tool(&send_message_tool_definition(), args.clone(), id).unwrap();
    assert_eq!(result["sent"], true);
    assert_eq!(counter.count(), 1);
    let after = durable_image(&path);
    let message = json!({"type":"text", "content":"one batch, two unique sources", "images":[
        {"url":resolved_first.url, "alt":"first", "width":1, "height":1},
        {"url":resolved_first.url, "alt":"repeat", "width":1, "height":1},
        {"url":resolved_second.url, "alt":"second", "width":1, "height":1},
    ]});
    assert_committed_image(&before, &after, &message, id);
    assert_eq!(result["messageId"], values(&after.rows[before.rows.len()..])[0]["id"]);
    assert!(fixture.deps().ack_obligations.store().get_checked(&fixture.agent_id).unwrap().is_none(),
        "the same real ACK obligation is fulfillable after confirmed success");
    assert_ne!(fs::read(&ack.path).unwrap(), ack.bytes);
    assert_terminal(&sink, id, PendingOutgoingMediaPhase::Committed);
    assert!(Arc::ptr_eq(&old, &pending_entry(&sink, id)), "a retained old Arc sees the terminal marker");
    assert_unknown(bridge.call_tool(&send_message_tool_definition(), args, id));
    assert_unknown(sink.resolve_attachment_source(&format!("{first_url}%00"), id));
    assert_unknown(counted.send_message(text_message("cannot reuse the committed media identity"), 4_001, id));
    sink.abort_attachment_custody(id);
    assert_terminal(&sink, id, PendingOutgoingMediaPhase::Committed);
    assert_eq!(counter.count(), 1);
    assert_eq!(durable_image(&path), after);
    assert_eq!(media_bytes(&fixture).len(), 2);
    for path in [&first, &second, &first_owned, &second_owned] { assert_eq!(fs::read(path).unwrap(), PNG); }
    assert_quiet(&fixture, &port);
    drop((old, bridge, counted));
    let weak = Arc::downgrade(&sink);
    drop(sink);
    assert!(weak.upgrade().is_none());
    let (root, agent) = close_for_reopen(fixture);
    for _ in 0..2 {
        let fresh = ChannelHostLaneFixture::open("msu-images-reopen", root.clone(), Some(agent.clone()));
        assert_eq!(durable_image(&path), after);
        for path in [&first, &second, &first_owned, &second_owned] { assert_eq!(fs::read(path).unwrap(), PNG); }
        let visible = fresh.entries();
        assert_eq!(visible.len(), before.rows.len() + 1);
        fresh.assert_no_followup(&visible);
        close_for_reopen(fresh);
    }
    fs::remove_dir_all(root).unwrap();
}

// Observation-only scheduling: production returns the busy error before this
// gate pauses the loser. Releasing the winner before the bridge handles that
// error makes an accidental build-error abort destructive and observable.
struct PauseAfterBusy {
    production: Arc<ProductionSendMessageSink>,
    reached: mpsc::Sender<()>,
    resume: Mutex<mpsc::Receiver<()>>,
    aborts: AtomicUsize,
}

impl SendMessageSink for PauseAfterBusy {
    fn resolve_attachment_source(&self, url: &str, id: &str) -> Result<ResolvedAttachmentSource, ProviderSessionError> {
        let result = self.production.resolve_attachment_source(url, id);
        if matches!(&result, Err(ProviderSessionError::ToolOutcomeUnknown(_))) {
            self.reached.send(()).unwrap();
            self.resume.lock().unwrap().recv_timeout(WAIT).expect("bounded loser scheduling gate");
        }
        result
    }

    fn read_media_dimensions(&self, url: &str) -> Option<(u32, u32)> { self.production.read_media_dimensions(url) }
    fn is_awaiting_user_selection(&self) -> bool { self.production.is_awaiting_user_selection() }
    fn abort_attachment_custody(&self, id: &str) {
        self.aborts.fetch_add(1, Ordering::AcqRel);
        self.production.abort_attachment_custody(id);
    }
    fn send_message(&self, message: Value, timestamp: u64, id: &str) -> Result<Option<String>, ProviderSessionError> {
        self.production.send_message(message, timestamp, id)
    }
}

#[test]
fn media_settlement_app_busy_entry_refuses_promptly_and_bridge_preserves_winner() {
    let mut fixture = ChannelHostLaneFixture::new("msu-busy-entry");
    let (port, _) = fixture.register_transport(TransportOutcome::Delivered, false);
    let (sink, ack) = take_sink_with_ack(&mut fixture);
    let media = PreparedMedia::materialize(&fixture, &sink, DIRECT_ID);
    let path = fixture.deps().session_workers.session_db_path(&fixture.agent_id).unwrap();
    let before = durable_image(&path);
    let files = media_bytes(&fixture);
    let entry = pending_entry(&sink, DIRECT_ID);
    let held = entry.lock().unwrap();
    let (reached_tx, reached_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let observer = Arc::new(PauseAfterBusy {
        production: sink.clone(), reached: reached_tx, resume: Mutex::new(resume_rx), aborts: AtomicUsize::new(0),
    });
    let production = sink.clone();
    let observed = observer.clone();
    let source_url = media.source_url.clone();
    let (done_tx, done_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let resolve = production.resolve_attachment_source(&source_url, DIRECT_ID);
        let send = production.send_message(text_message("busy send"), 5_000, DIRECT_ID);
        production.abort_attachment_custody(DIRECT_ID);
        let unrelated = production.resolve_attachment_source("https://fixture.invalid/unrelated.png", "msu-unrelated-entry");
        production.abort_attachment_custody("msu-unrelated-entry");
        let bridge = SendMessageToolBridge::new(Arc::new(UnusedCompactionProviderPorts), observed);
        let bridge_result = bridge.call_tool(&send_message_tool_definition(),
            json!({"type":"attachment", "url":source_url, "channel":ADDRESS}), DIRECT_ID);
        done_tx.send((resolve, send, unrelated, bridge_result)).unwrap();
    });
    let reached = reached_rx.recv_timeout(CLEANUP_WAIT);
    // Always release before reporting a failed promptness assertion, so a
    // regressed blocking implementation is not left with a test-owned lock.
    drop(held);
    let _ = resume_tx.send(());
    let done = done_rx.recv_timeout(WAIT).expect("owned busy-operation worker must finish");
    worker.join().unwrap();
    assert!(reached.is_ok(), "same-entry operations and unrelated entry must finish before the held lock is released: {reached:?}");
    assert_unknown(done.0);
    assert_unknown(done.1);
    assert_eq!(done.2.unwrap().url, "https://fixture.invalid/unrelated.png");
    assert_unknown(done.3);
    assert_eq!(observer.aborts.load(Ordering::Acquire), 0,
        "the actual tool bridge must not abort an unknown build result after the winner becomes available");
    assert_terminal(&sink, "msu-unrelated-entry", PendingOutgoingMediaPhase::Aborted);
    assert!(Arc::ptr_eq(&entry, &pending_entry(&sink, DIRECT_ID)));
    assert_eq!(entry.try_lock().unwrap().phase, PendingOutgoingMediaPhase::Collecting);
    assert_eq!(durable_image(&path), before);
    assert_eq!(media_bytes(&fixture), files);
    media.assert_bytes(true);
    ack.assert_retained(&fixture.root, &fixture.agent_id);
    assert_quiet(&fixture, &port);
    // The winner's already-prepared media can still settle through the owner.
    let counter = SendMessageDeliveryCounter::default();
    let counted = CountingSendMessageSink::new(sink.clone(), counter.clone());
    assert!(counted.send_message(media.message(false), 5_001, DIRECT_ID).unwrap().is_some());
    assert_eq!(counter.count(), 1);
    assert_committed_image(&before, &durable_image(&path), &media.message(false), DIRECT_ID);
    assert_terminal(&sink, DIRECT_ID, PendingOutgoingMediaPhase::Committed);
    media.assert_bytes(true);
    drop((entry, observer, counted));
    let weak = Arc::downgrade(&sink);
    drop(sink);
    assert!(weak.upgrade().is_none());
    fixture.close();
}

#[test]
fn media_settlement_app_poisoned_entry_and_sealed_unwind_retain_for_recovery() {
    for seal_before_unwind in [false, true] {
        let mut fixture = ChannelHostLaneFixture::new("msu-poison-unwind");
        let (port, _) = fixture.register_transport(TransportOutcome::Delivered, false);
        let (sink, ack) = take_sink_with_ack(&mut fixture);
        let media = PreparedMedia::materialize(&fixture, &sink, DIRECT_ID);
        let path = fixture.deps().session_workers.session_db_path(&fixture.agent_id).unwrap();
        let before = durable_image(&path);
        let entry = pending_entry(&sink, DIRECT_ID);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let mut pending = entry.lock().unwrap();
            if seal_before_unwind { pending.begin_owner_settlement().unwrap(); }
            panic!("controlled unwind with the actual media entry guard held");
        })).is_err());
        assert!(entry.is_poisoned());
        assert_unknown(sink.resolve_attachment_source(&format!("{}%00", media.source_url), DIRECT_ID));
        assert_unknown(sink.send_message(media.message(true), 6_000, DIRECT_ID));
        sink.abort_attachment_custody(DIRECT_ID);
        {
            let retained = entry.lock().unwrap_or_else(|poison| poison.into_inner());
            assert_eq!(retained.phase, PendingOutgoingMediaPhase::OwnerSettlementPending);
            assert!(retained.custody.is_owner_settlement_pending());
            assert_eq!(retained.media_work_id, before.admission.admitted_media_work_id);
        }
        assert_eq!(durable_image(&path), before, "this is an unwind control, not a claimed SQL fault");
        media.assert_bytes(true);
        ack.assert_retained(&fixture.root, &fixture.agent_id);
        assert_quiet(&fixture, &port);
        let entry_weak = Arc::downgrade(&entry);
        drop(entry);
        let sink_weak = Arc::downgrade(&sink);
        drop(sink);
        assert!(sink_weak.upgrade().is_none() && entry_weak.upgrade().is_none());
        media.assert_bytes(true);
        assert_eq!(durable_image(&path), before);
        let (root, agent) = close_for_reopen(fixture);
        let expected = DurableImage { rows: before.rows, admission: released(before.admission) };
        assert_two_fresh_owners(&root, &agent, &path, &expected, &media, false, &ack);
        fs::remove_dir_all(root).unwrap();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalImage {
    root: Value,
    blobs: Vec<(String, Vec<u8>)>,
    terminals: Vec<(String, Vec<u8>)>,
    journal: Option<Vec<u8>>,
    mirror_files: BTreeMap<PathBuf, Vec<u8>>,
    provider_files: BTreeMap<PathBuf, Vec<u8>>,
}

fn canonical_image(root: &Path, agent: &str, db_path: &Path, blob_path: &Path) -> CanonicalImage {
    // Read the durable images independently even after the production owner
    // has been retired. A lenient closed-owner getter is not an absence proof.
    let db = reader(db_path);
    let metadata: String = db.query_row("SELECT value FROM kv WHERE key = 'metadata'", [], |row| row.get(0)).unwrap();
    assert_eq!(metadata.len() % 2, 0);
    let metadata = metadata.as_bytes().chunks_exact(2).map(|pair| {
        u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()
    }).collect::<Vec<_>>();
    let metadata: Value = serde_json::from_slice(&metadata).unwrap();
    let mut terminals = db.prepare("SELECT key, CAST(value AS BLOB) FROM kv WHERE key LIKE 'original-routed-terminal:%' ORDER BY key").unwrap();
    let terminals = terminals.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap().map(Result::unwrap).collect();
    let blobs = if blob_path.exists() {
        let db = reader(blob_path);
        let mut statement = db.prepare("SELECT id, data FROM blobs ORDER BY id").unwrap();
        statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap().map(Result::unwrap).collect()
    } else {
        // A never-written blob store is a valid pre-invocation baseline.
        // Empty-store initialization does not constitute canonical history.
        Vec::new()
    };
    CanonicalImage {
        root: metadata.get("latestRootBlobId").expect("durable canonical root field").clone(),
        blobs, terminals,
        journal: optional_bytes(&root.join("transcripts").join(agent).join(format!("{agent}.jsonl"))),
        mirror_files: file_bytes(&root.join("agent-store").join(agent)),
        provider_files: file_bytes(&root.join("runner-provider-checkpoints")),
    }
}

fn assert_live_completed_error(events: &[Value], agent: &str) {
    let calls = events.iter().filter(|event| event["type"] == "agent-tool-call").collect::<Vec<_>>();
    assert_eq!(calls.len(), 2, "one actual initial/completed pair, with no successful tool result");
    for (event, phase) in [(calls[0], "initial"), (calls[1], "completed")] {
        assert_eq!(event["agentId"], agent);
        assert_eq!(event["tool"], "SendMessage");
        assert_eq!(event["toolCallId"], GENERATED_ID);
        assert_eq!(event["phase"], phase);
        assert_eq!(event["encoding"], "agent.v1.protobuf");
    }
    let wire = serde_json::from_value::<Vec<u8>>(calls[1]["wire"].clone()).unwrap();
    let call = fields(&wire);
    assert_eq!(call.get(&57), Some(&Field::Bytes(GENERATED_ID.as_bytes())));
    let Some(Field::Bytes(send)) = call.get(&55) else { panic!("actual SendMessage completed wire"); };
    let send = fields(send);
    let Some(Field::Bytes(result)) = send.get(&2) else { panic!("actual completed result"); };
    let result = fields(result);
    assert_eq!(result.len(), 1);
    assert!(!result.contains_key(&1), "no fabricated SendMessageSuccess payload");
    let Some(Field::Bytes(error)) = result.get(&2) else { panic!("actual SendMessageError payload"); };
    let error = fields(error);
    let Some(Field::Bytes(detail)) = error.get(&1) else { panic!("actual error text field"); };
    assert!(!detail.is_empty());
}

fn generated_lost_ack_child(root: PathBuf) -> Value {
    let mut fixture = ChannelHostLaneFixture::open("msu-generated", root.clone(), None);
    let (port, _) = fixture.register_transport(TransportOutcome::Delivered, false);
    let sessions = Arc::clone(&fixture.deps().session_workers);
    let runtime = Arc::clone(&fixture.deps().transcript_runtime);
    let registry = fixture.deps().runner_registry.clone();
    let args = json!({"agentId":fixture.agent_id, "sessionKey":fixture.session_key, "streamId":STREAM,
        "clientNonce":"msu-generated-original-nonce", "prompt":PROMPT});
    let accepted = runtime.accept_routed_send(&args, |accepted| {
        persist_accepted_send_prompt_context(&sessions, &args, accepted)
            .map_err(|error| ProductionSendError::Internal(format!("{error:?}")))
    }).unwrap();
    assert!(accepted.context.accepted_durably && !accepted.context.duplicate);
    let control = runtime.capture_routed_turn_start(&fixture.agent_id, STREAM).unwrap();
    let cancellation = registry.register_routed_provider_with_cancellation(
        &fixture.agent_id, STREAM, false, control.cancellation(),
    ).unwrap();
    assert!(registry.mark_routed_provider_dispatched(STREAM));
    let admission = sessions.read_session_work_admission(&fixture.agent_id).unwrap().unwrap();
    let original = runtime.bind_original_turn_model(&fixture.agent_id, STREAM, &admission, MODEL).unwrap().unwrap();
    assert_eq!(accepted.context.user_message_id.as_deref(), Some(original.user_message_id.as_str()));
    let checkpoint: Arc<dyn AgentStateCheckpointSink> = fixture.deps().host_runner_composition
        .compose_production_checkpoint_sink(&sessions, &root, &fixture.agent_id, false,
            Arc::new(|_| {}), Arc::new(|| Ok(true))).unwrap().unwrap();
    assert!(checkpoint.committed_original_turn_observations().unwrap().is_empty());
    let store = sessions.open_agent_store_owner(&fixture.agent_id).unwrap();
    let blob_path = store.blob_db_path.clone();
    let (sink, ack) = take_sink_with_ack(&mut fixture);
    // Prepare through the actual sink, then let the generated producer resolve
    // the identical source through its collecting cache. This pins the exact
    // real owner/fence that the borrowed fault lease must consume.
    let media = PreparedMedia::materialize(&fixture, &sink, GENERATED_ID);
    let owner = sessions.open_agent_db_owner(&fixture.agent_id).unwrap();
    let db_path = owner.db_path().to_path_buf();
    let before = durable_image(&db_path);
    let history = canonical_image(&root, &fixture.agent_id, &db_path, &blob_path);
    assert!(history.terminals.is_empty());
    let fault = owner.arm_media_settlement_fault_for_test(
        &before.admission.session_key, &before.admission.lifecycle_revision,
        &format!("send-media:{GENERATED_ID}"), MediaSettlementTestFault::LoseCommitAcknowledgement,
    ).unwrap();
    assert!(!fault.was_claimed() && !fault.was_applied());
    let publications = fixture.deps().events.subscribe();
    let observed_events = Arc::new(Mutex::new(Vec::<Value>::new()));
    let captured = observed_events.clone();
    let observation = TurnObservation::shared(&fixture.agent_id,
        Some(Arc::new(move |event| captured.lock().unwrap().push(event))));
    let counter = SendMessageDeliveryCounter::default();
    let counted: Arc<dyn SendMessageSink> = Arc::new(CountingSendMessageSink::new(sink.clone(), counter.clone()));
    let retries = Arc::new(AtomicUsize::new(0));
    let retry_count = retries.clone();
    let composition = TurnAgentComposition::new(
        RoutedProvider::Fabushi, Arc::new(UnusedCompactionProviderPorts),
        RunnerRequestContextSnapshot { context: mahayana_host_runtime::host_request_context::HostRequestContext {
            os_version:"media-settlement-unknown".into(), shell:None, time_zone:Some("UTC".into()),
            transcripts_folder:root.join("transcripts").to_string_lossy().into_owned(), user_full_name:None,
        }, rules:None }, cancellation.clone(),
        Arc::new(ProductionRoutedProviderCheckpointStore::new(&root, &fixture.agent_id, STREAM)),
    ).with_fabushi_model(MODEL).with_send_message_sink(counted.clone())
        .with_retry_sink(Arc::new(move |_| { retry_count.fetch_add(1, Ordering::AcqRel); }))
        .with_observation(observation.clone());
    let successes = Arc::new(AtomicUsize::new(0));
    let success_count = successes.clone();
    let mut runner = create_production_runner(composition,
        ProductionTurnAgentBuildBindings::new(ProductionTurnAgentStaticConfig {
            model_id:MODEL.into(), agent_token_limit:SAND_AGENT_TOKEN_LIMIT,
            conversation_id:fixture.agent_id.clone(), is_box_scoped_subagent:false,
            is_subagent_runner:false, is_shared_room_runner:false,
            sand_send_message_delivery_owed:false, transcripts_folder_available:true,
        }, Arc::new(|| None), Arc::new(|_, _, _| panic!("MSU fixture must not summarize"))),
        ProductionTurnAgentLifecycleBindings::new(&fixture.agent_id, STREAM)
            .with_profile_announcement_commit(Some(Arc::new(move || { success_count.fetch_add(1, Ordering::AcqRel); }))),
        Some(checkpoint.clone()), Arc::new(AtomicBool::new(false)),
        Arc::new(Mutex::new(SubagentRuntime::default())), Some(observation), None, None, None,
    );
    let mut deltas = Vec::new();
    let result = runner.run_routed_provider_with_options(&root,
        &[ProviderMessage { role:"user".into(), content:PROMPT.into() }],
        TurnRunOptions { inference_request_id:Some(STREAM.into()), message_id:Some(original.user_message_id.clone()),
            recent_message_text:Some(PROMPT.into()), turn_started_at_ms:Some(1), ..TurnRunOptions::default() },
        &mut |delta, _| deltas.push(delta.to_string()),
    );
    assert!(result.is_err(), "the generated consumer cannot invent success from the durable send row");
    assert!(deltas.is_empty());
    assert!(fault.was_claimed());
    assert!(fault.was_applied(), "this generated invocation must lose the real successful COMMIT acknowledgement");
    assert!(owner.is_closed());
    assert_eq!(counter.count(), 0);
    assert_eq!(retries.load(Ordering::Acquire), 0);
    assert_eq!(successes.load(Ordering::Acquire), 0);
    let finished = runner.last_finished().expect("actual generated shell terminal");
    assert_eq!(finished.owner.request_id, STREAM);
    assert!(finished.owner.generation > 0);
    assert!(matches!(finished.outcome, TerminalOutcome::Failed { .. }));
    assert_eq!(runner.last_tool_effect_custody(), Some(&ToolEffectCustodyObservation::Known {
        run_generation:finished.owner.generation,
        snapshot:ToolEffectCustodySnapshot { started_revision:1, checkpoint_revision:0, pending_effects:0,
            unknown_result:Some(ToolEffectUnknownResult::DownstreamOutcomeUnknown) },
    }), "the actual worker/guard must retain sticky downstream-unknown custody");
    assert!(checkpoint.committed_original_turn_observations().is_err());
    assert_live_completed_error(&observed_events.lock().unwrap(), &fixture.agent_id);
    let after = durable_image(&db_path);
    assert_committed_image(&before, &after, &media.message(true), GENERATED_ID);
    assert_eq!(canonical_image(&root, &fixture.agent_id, &db_path, &blob_path), history,
        "unknown cannot persist a new canonical tool/error/final step, Journal or provider checkpoint");
    assert_unknown_entry(&sink, GENERATED_ID, &before.admission, &media);
    ack.assert_bound(&sink);
    ack.assert_retained(&root, &fixture.agent_id);
    assert!(matches!(publications.try_recv(), Err(mpsc::TryRecvError::Empty)), "no successful App transcript publication");
    assert_quiet(&fixture, &port);
    let mut custody = ToolEffectCustodyReport::new(STREAM);
    custody.record(runner.last_tool_effect_custody());
    assert!(custody.requires_retention());
    // The Responses/shell layer may map the final error to Tool. The direct
    // cases assert the actual sink's typed error; custody is authoritative here.
    let terminal = crate::RoutedProviderTerminal::capture(STREAM, false, &cancellation,
        result, counter.count(), false, custody);
    assert_eq!(terminal.event["payload"]["type"], "failed");
    assert!(terminal.persist_original_turn(&sessions, &original, checkpoint.as_ref()).is_err());
    assert_eq!(canonical_image(&root, &fixture.agent_id, &db_path, &blob_path), history);
    assert_unknown(sink.resolve_attachment_source(&media.source_url, GENERATED_ID));
    assert_unknown(counted.send_message(media.message(true), 7_001, GENERATED_ID));
    sink.abort_attachment_custody(GENERATED_ID);
    assert_unknown_entry(&sink, GENERATED_ID, &before.admission, &media);
    assert_eq!(durable_image(&db_path), after);
    assert_eq!(counter.count(), 0);
    media.assert_bytes(true);
    registry.finish_routed_provider(STREAM);
    runtime.settle_routed_turn_control(&control, started_at_ms()).unwrap().unwrap();
    assert_eq!(registry.active_count(), 0);
    assert!(!runtime.is_agent_running(&fixture.agent_id));
    drop(fault);
    drop((runner, counted, checkpoint, terminal, control, store, owner));
    let weak = Arc::downgrade(&sink);
    drop(sink);
    let deadline = std::time::Instant::now() + CLEANUP_WAIT;
    while weak.strong_count() != 0 && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(2));
    }
    assert!(weak.upgrade().is_none(), "every generated composition/observer sink Arc is gone");
    media.assert_bytes(true);
    assert_eq!(durable_image(&db_path), after);
    let (root, agent) = fixture.stop_for_reopen();
    sessions.shutdown_with_checkpoint();
    assert_eq!(sessions.active_worker_count(), 0);
    drop((sessions, runtime, registry));
    fs::remove_file(root.join("responses-credential.json")).unwrap();
    for reopen in 0..2 {
        let fresh = ChannelHostLaneFixture::open("msu-generated-fresh", root.clone(), Some(agent.clone()));
        assert_eq!(durable_image(&db_path), after, "fresh owner {reopen} cannot replay a lost-ACK send");
        assert_eq!(canonical_image(&root, &agent, &db_path, &blob_path), history);
        assert!(reconcile_completed_original_turn(&fresh.deps().session_workers, &original).unwrap().is_none());
        let fresh_checkpoint = fresh.deps().host_runner_composition.compose_production_checkpoint_sink(
            &fresh.deps().session_workers, &root, &agent, false, Arc::new(|_| {}), Arc::new(|| Ok(true)),
        ).unwrap().unwrap();
        assert!(fresh_checkpoint.committed_original_turn_observations().unwrap().is_empty());
        media.assert_bytes(true);
        ack.assert_retained(&root, &agent);
        let visible = values(&after.rows).into_iter()
            .filter(|entry| entry["kind"] != "outgoing-media-settlement").collect::<Vec<_>>();
        assert_eq!(fresh.entries(), visible);
        fresh.assert_no_followup(&visible);
        fresh.assert_no_failure();
        assert!(matches!(fresh.lane_observations.try_recv(), Err(mpsc::TryRecvError::Empty)));
        drop(fresh_checkpoint);
        close_for_reopen(fresh);
        assert_eq!(durable_image(&db_path), after);
        assert_eq!(canonical_image(&root, &agent, &db_path, &blob_path), history);
    }
    json!({"faultClaimed":true, "faultApplied":true, "sentCount":counter.count(),
        "retryCount":retries.load(Ordering::Acquire), "successfulHooks":successes.load(Ordering::Acquire),
        "freshOwners":2, "addedSqlRows":after.rows.len() - before.rows.len(),
        "canonicalHistoryUnchanged":true, "ackRetained":true, "mediaRetained":true})
}

#[test]
fn media_settlement_app_generated_runner_lost_ack_retains_unknown_custody() {
    if let Some(path) = std::env::var_os(CHILD_REQUEST_ENV) {
        let request: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_ne!(request["parentPid"].as_u64().unwrap(), u64::from(std::process::id()));
        let root = PathBuf::from(request["root"].as_str().unwrap());
        assert!(root.is_absolute() && PathBuf::from(path).starts_with(&root));
        let proof = generated_lost_ack_child(root.clone());
        let mut receipt = fs::OpenOptions::new().write(true).create_new(true).open(root.join("completed.json")).unwrap();
        receipt.write_all(json!({"token":request["token"], "parentPid":request["parentPid"],
            "pid":std::process::id(), "proof":proof}).to_string().as_bytes()).unwrap();
        receipt.sync_all().unwrap();
        return;
    }
    let root = std::env::temp_dir().join(format!("fabushi-media-settlement-generated-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    fs::write(root.join("settings.json"), b"{}").unwrap();
    let token = uuid::Uuid::new_v4().to_string();
    let request_path = root.join("request.json");
    fs::write(&request_path, json!({"token":token, "parentPid":std::process::id(), "root":root}).to_string()).unwrap();
    let mut responses = Responses::with_events(&root, |root, ordinal| {
        if ordinal != 0 { return Err(format!("unknown media must not produce provider continuation/retry {ordinal}")); }
        Ok(vec![json!({"type":"response.completed", "response":{
            "id":"msu-one-actual-attachment", "output":[{"type":"function_call", "name":"SendMessage",
                "call_id":GENERATED_ID, "arguments":json!({"type":"attachment",
                    "url":file_url_for_path(root.join(SOURCE_NAME)).unwrap(),
                    "alt":"retained MSU attachment", "channel":ADDRESS}).to_string()}],
        }})])
    });
    let stdout_path = root.join("child.stdout");
    let stderr_path = root.join("child.stderr");
    let mut child = OwnedChild(Some(Command::new(std::env::current_exe().unwrap())
        .args(["--exact", GENERATED_TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD_REQUEST_ENV, &request_path)
        .env("FABUSHI_RESPONSES_URL", &responses.endpoint)
        .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &responses.credential)
        .env("FABUSHI_RESPONSES_MODEL", MODEL)
        .env("NO_PROXY", "127.0.0.1,localhost").env("no_proxy", "127.0.0.1,localhost")
        .env_remove("HTTP_PROXY").env_remove("HTTPS_PROXY").env_remove("ALL_PROXY")
        .env_remove("http_proxy").env_remove("https_proxy").env_remove("all_proxy")
        .current_dir(&root).stdin(Stdio::null())
        .stdout(Stdio::from(fs::File::create(&stdout_path).unwrap()))
        .stderr(Stdio::from(fs::File::create(&stderr_path).unwrap()))
        .spawn().expect("isolated actual generated Runner process")));
    let pid = child.0.as_ref().unwrap().id();
    let child_result = child.wait();
    drop(child);
    let server_result = responses.finish();
    let stdout = fs::read_to_string(stdout_path).unwrap();
    let stderr = fs::read_to_string(stderr_path).unwrap();
    assert!(child_result.as_ref().is_ok_and(|status| status.success()) && server_result.is_ok(),
        "MSU child={child_result:?} Responses={server_result:?}\n{stdout}\n{stderr}\nretained fixture={}", root.display());
    let receipt: Value = serde_json::from_slice(&fs::read(root.join("completed.json")).unwrap()).unwrap();
    assert_eq!(receipt["token"], token);
    assert_eq!(receipt["pid"], pid);
    assert_eq!(receipt["parentPid"], std::process::id());
    assert_eq!(receipt["proof"], json!({"faultClaimed":true, "faultApplied":true, "sentCount":0,
        "retryCount":0, "successfulHooks":0, "freshOwners":2, "addedSqlRows":2,
        "canonicalHistoryUnchanged":true, "ackRetained":true, "mediaRetained":true}));
    let requests = responses.requests.lock().unwrap();
    assert_eq!(requests.len(), 1, "unknown prevents both replay and a successful provider continuation");
    assert_eq!(requests[0]["model"], MODEL);
    drop(requests);
    eprintln!("MSU generated producer_pid={pid} real_commit_lost_ack=true sends=0 retries=0 fresh_owners=2");
    drop(responses);
    fs::remove_dir_all(root).unwrap();
}
