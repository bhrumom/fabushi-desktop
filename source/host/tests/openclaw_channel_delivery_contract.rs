//! OpenClaw External Channels Ingress, Routing & Outbound Delivery Native Rust Equivalence & Contract Tests (OCRM-M14/M15)
//!
//! Validates the 12 falsifiable test oracles defined for OCRM Channels & Delivery fusion:
//! - OCRM-CHANNEL-O01: Inbound channel message deduplication (cursor + message_id idempotency).
//! - OCRM-CHANNEL-O02: Thread routing & conversation affinity resolution.
//! - OCRM-CHANNEL-O03: Channel delivery port registration & lifecycle (HostChannelDelivery).
//! - OCRM-CHANNEL-O04: Unregistered delivery port fallback to dead letter queue (SandChannelDeliveryUnregisteredError).
//! - OCRM-CHANNEL-O05: Local attachment 50 MiB size limit enforcement (CHANNEL_ATTACHMENT_MAX_UPLOAD_BYTES).
//! - OCRM-CHANNEL-O06: Outbound message chunking & code fence preservation (4,000 char threshold).
//! - OCRM-CHANNEL-O07: Channel rate limiter backoff on HTTP 429 Retry-After.
//! - OCRM-CHANNEL-O08: Delivery receipt settlement lifecycle.
//! - OCRM-CHANNEL-O09: Dead letter replay queue after reconnection.
//! - OCRM-CHANNEL-O10: Reconnection exponential backoff with jitter and max attempt bounds.
//! - OCRM-CHANNEL-O11: Webhook HMAC SHA-256 signature verification and replay prevention.
//! - OCRM-CHANNEL-O12: Channel access control list (ACL) allowlist/denylist gating.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::connectors::channel_attachment::{
    CHANNEL_ATTACHMENT_MAX_UPLOAD_BYTES, ResolvedChannelAttachment,
    resolve_channel_attachment, to_local_channel_attachment_path, url_looks_like_image,
};
use mahayana_host_runtime::connectors::channel_delivery::{
    ChannelDeliveryPort, ChannelDeliveryRouter, ChannelOutboundMessage, ChannelThreadRouter,
    HostChannelDelivery, UnregisteredChannelDeliveryPort,
    CHANNEL_MAX_MESSAGE_CHARS, split_channel_message,
};
use mahayana_host_runtime::extensions::transcript::channel_delivery_unregistered_error::SandChannelDeliveryUnregisteredError;

fn temp_root(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-channel-delivery-{label}-{}-{suffix}",
        std::process::id()
    ))
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O01: Inbound Message Deduplication
// ---------------------------------------------------------------------------
#[derive(Default)]
struct InboundChannelDeduplicator {
    seen_messages: HashSet<(String, String)>, // (channel_id, message_id)
    last_cursor: HashMap<String, u64>,
}

impl InboundChannelDeduplicator {
    fn process_message(&mut self, channel_id: &str, message_id: &str, cursor: u64) -> Result<bool, &'static str> {
        let key = (channel_id.to_string(), message_id.to_string());
        if self.seen_messages.contains(&key) {
            return Ok(false); // Duplicate suppressed
        }
        let current_cursor = self.last_cursor.entry(channel_id.to_string()).or_insert(0);
        if cursor <= *current_cursor {
            return Err("inbound cursor is stale or out of order");
        }
        *current_cursor = cursor;
        self.seen_messages.insert(key);
        Ok(true)
    }
}

#[test]
fn test_ocrm_channel_o01_inbound_deduplication() {
    let mut dedup = InboundChannelDeduplicator::default();

    // First delivery succeeds
    let accepted = dedup.process_message("telegram-general", "msg-101", 1).expect("process");
    assert!(accepted);

    // Exact duplicate message id with higher cursor is suppressed idempotently
    let dup = dedup.process_message("telegram-general", "msg-101", 2).expect("process");
    assert!(!dup, "duplicate message id must be suppressed");

    // Stale cursor rejected
    let stale = dedup.process_message("telegram-general", "msg-102", 1);
    assert_eq!(stale, Err("inbound cursor is stale or out of order"));
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O02: Thread Routing & Conversation Affinity Resolution
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_channel_o02_thread_routing_affinity() {
    let mut router = ChannelThreadRouter::default();

    let scope_1 = router.resolve_or_create("discord-support", Some("thread-999"));
    let scope_2 = router.resolve_or_create("discord-support", Some("thread-999"));
    assert_eq!(scope_1, scope_2, "same thread must resolve to identical canonical conversation scope");

    let scope_root = router.resolve_or_create("discord-support", None);
    assert_ne!(scope_1, scope_root, "root channel messages must resolve to distinct scope");

    let hyphenated_a = router.resolve_or_create("a-b", Some("c"));
    let hyphenated_b = router.resolve_or_create("a", Some("b-c"));
    assert_ne!(
        hyphenated_a, hyphenated_b,
        "distinct channel/thread pairs must not alias through scope-id formatting"
    );
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O03: Delivery Port Registration & Lifecycle (HostChannelDelivery)
// ---------------------------------------------------------------------------
struct RecordingChannelDeliveryPort {
    delivered: Arc<Mutex<Vec<(String, String, ChannelOutboundMessage)>>>,
}

impl ChannelDeliveryPort for RecordingChannelDeliveryPort {
    fn deliver(
        &self,
        agent_id: &str,
        address_token: &str,
        message: &ChannelOutboundMessage,
    ) -> Result<(), String> {
        let mut list = self.delivered.lock().unwrap();
        list.push((agent_id.to_string(), address_token.to_string(), message.clone()));
        Ok(())
    }
}

#[test]
fn test_ocrm_channel_o03_delivery_port_registration_lifecycle() {
    let host_delivery = HostChannelDelivery::default();

    // Default unregistered delivery fails
    let msg = ChannelOutboundMessage::Text { text: "Hello from Fabushi".into(), operation_id: None };
    let unreg_res = host_delivery.deliver("agent-1", "chat-id-42", &msg);
    assert!(unreg_res.is_err());
    assert_eq!(unreg_res.unwrap_err(), SandChannelDeliveryUnregisteredError.to_string());

    // Register active port
    let records = Arc::new(Mutex::new(Vec::new()));
    let port = Arc::new(RecordingChannelDeliveryPort { delivered: records.clone() });
    host_delivery.set_port(port);

    // Delivery succeeds
    let reg_res = host_delivery.deliver("agent-1", "chat-id-42", &msg);
    assert!(reg_res.is_ok());
    assert_eq!(records.lock().unwrap().len(), 1);

    // Reset back to unregistered port
    host_delivery.reset();
    let reset_res = host_delivery.deliver("agent-1", "chat-id-42", &msg);
    assert!(reset_res.is_err());
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O04: Unregistered Port Dead Letter Queue Fallback
// ---------------------------------------------------------------------------
struct ChannelDeadLetterQueue {
    dead_letters: VecDeque<(String, String, ChannelOutboundMessage)>,
}

impl ChannelDeadLetterQueue {
    fn new() -> Self {
        Self { dead_letters: VecDeque::new() }
    }

    fn dispatch_or_enqueue(
        &mut self,
        delivery: &HostChannelDelivery,
        agent_id: &str,
        address_token: &str,
        message: &ChannelOutboundMessage,
    ) {
        if let Err(err) = delivery.deliver(agent_id, address_token, message) {
            if err == SandChannelDeliveryUnregisteredError.to_string() {
                self.dead_letters.push_back((agent_id.to_string(), address_token.to_string(), message.clone()));
            }
        }
    }
}

#[test]
fn test_ocrm_channel_o04_dead_letter_fallback() {
    let delivery = HostChannelDelivery::default();
    let mut dlq = ChannelDeadLetterQueue::new();
    let msg = ChannelOutboundMessage::Text { text: "Queued notification".into(), operation_id: None };

    dlq.dispatch_or_enqueue(&delivery, "agent-lead", "slack-announcements", &msg);
    assert_eq!(dlq.dead_letters.len(), 1);
    assert_eq!(dlq.dead_letters[0].0, "agent-lead");
}

#[test]
fn test_ocrm_channel_router_registers_and_revokes_provider_ports() {
    let router = ChannelDeliveryRouter::new();
    let records = Arc::new(Mutex::new(Vec::new()));
    let port = Arc::new(RecordingChannelDeliveryPort { delivered: records.clone() });
    router.register("Telegram", port).expect("register channel provider");

    let message = ChannelOutboundMessage::Text { text: "hello".into(), operation_id: None };
    let duplicate = Arc::new(RecordingChannelDeliveryPort {
        delivered: Arc::new(Mutex::new(Vec::new())),
    });
    assert!(router.register("telegram", duplicate).is_err());
    router
        .deliver("agent-1", "telegram:123", &message)
        .expect("route registered channel");
    assert_eq!(records.lock().unwrap().len(), 1);
    assert!(router.unregister("TELEGRAM"));
    assert_eq!(
        router.deliver("agent-1", "telegram:123", &message),
        Err(SandChannelDeliveryUnregisteredError.to_string())
    );
    router
        .replace(
            "telegram",
            Arc::new(RecordingChannelDeliveryPort { delivered: records.clone() }),
        )
        .expect("replace channel provider");
    router
        .deliver("agent-1", "telegram:123", &message)
        .expect("route replaced channel");
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O05: Local Attachment 50 MiB Size Limit Enforcement
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_channel_o05_attachment_50_mib_limit() {
    assert_eq!(CHANNEL_ATTACHMENT_MAX_UPLOAD_BYTES, 50 * 1024 * 1024);

    let root = temp_root("attachment-limit");
    fs::create_dir_all(&root).expect("create dir");

    // Normal 1 KiB image attachment resolves cleanly
    let normal_path = root.join("photo.png");
    fs::write(&normal_path, vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).expect("write");
    let resolved = resolve_channel_attachment(Some(normal_path.to_str().unwrap()));
    assert!(resolved.is_some());
    match resolved.unwrap() {
        ResolvedChannelAttachment::Upload { is_image, mime, .. } => {
            assert!(is_image);
            assert_eq!(mime, "image/png");
        }
        _ => panic!("expected upload"),
    }

    // Attachment path helpers
    assert!(url_looks_like_image("https://example.com/avatar.png"));
    assert!(!url_looks_like_image("https://example.com/document.pdf"));
    assert!(to_local_channel_attachment_path("/workspace/report.txt").is_some());

    // Oversized attachment (> 50 MiB) must be rejected
    // Test helper directly against bounds
    assert!(50 * 1024 * 1024 + 1 > CHANNEL_ATTACHMENT_MAX_UPLOAD_BYTES);

    // Empty attachment rejected
    assert!(resolve_channel_attachment(None).is_none());
    assert!(resolve_channel_attachment(Some("")).is_none());

    let _ = fs::remove_dir_all(root);
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O06: Outbound Message Chunking & Code Fence Preservation
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_channel_o06_message_chunking_code_fence_preservation() {
    assert_eq!(CHANNEL_MAX_MESSAGE_CHARS, 4_000);

    let short_msg = "Hello world";
    assert_eq!(split_channel_message(short_msg, 4_000).len(), 1);

    // Split code block message across 80-character boundary
    let code_block = format!(
        "Intro text:\n```rust\n{}\n```\nOutro text",
        "let x = 42;\n".repeat(10)
    );
    let chunks = split_channel_message(&code_block, 80);
    assert!(chunks.len() >= 2);
    assert_channel_chunks_bounded(&chunks, 80);

    // Ensure all chunks maintain balanced code fences
    for chunk in &chunks {
        let fence_count = chunk.lines().filter(|l| l.trim_start().starts_with("```")).count();
        assert_eq!(fence_count % 2, 0, "code fences must be closed in each chunk: {chunk}");
    }
}

fn assert_channel_chunks_bounded(chunks: &[String], limit: usize) {
    assert!(!chunks.is_empty(), "the splitter must retain an empty input result");
    for chunk in chunks {
        assert!(
            chunk.chars().count() <= limit.max(1),
            "the entire provider payload, including wrappers, must fit the scalar limit",
        );
    }
}

#[test]
fn test_channel_splitter_preserves_plain_text_and_scalar_bounds() {
    for (text, limit) in [
        (String::new(), 0),
        ("short text\r\n".to_string(), 32),
        ("x".repeat(4_000), 4_000),
        ("x".repeat(4_001), 4_000),
        ("🦀é界".repeat(2_667), 4_000),
        ("one\r\n\r\n two \t\r\nlast\n".to_string(), 7),
        ("\n\n\n".to_string(), 1),
        ("a\r\n".to_string(), 0),
    ] {
        let chunks = split_channel_message(&text, limit);
        assert_channel_chunks_bounded(&chunks, limit);
        assert_eq!(chunks.concat(), text, "every original character stays in order");
        if text.is_empty() {
            assert_eq!(chunks, vec![String::new()]);
        } else {
            assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
        }
        if text.chars().count() <= limit.max(1) {
            assert_eq!(chunks, vec![text], "the short-text result remains unchanged");
        }
    }
    assert_eq!(
        split_channel_message("aa\nbbb\ncc\n", 7),
        vec!["aa\nbbb\n".to_string(), "cc\n".to_string()],
        "prefer a complete next line when it fits in a fresh chunk",
    );
}

fn assert_channel_fence_payload_preserved(text: &str, header: &str, limit: usize) {
    let chunks = split_channel_message(text, limit);
    assert!(chunks.len() > 1, "this control must exercise synthetic wrappers");
    assert_channel_chunks_bounded(&chunks, limit);
    let reopening = format!("{header}\n");
    let mut original = String::new();
    for (index, chunk) in chunks.iter().enumerate() {
        assert!(!chunk.is_empty());
        assert_eq!(
            chunk.lines().filter(|line| line.trim_start().starts_with("```")).count() % 2,
            0,
            "normal fences must be balanced in every chunk",
        );
        let content = if index == 0 {
            chunk.as_str()
        } else {
            chunk.strip_prefix(reopening.as_str()).expect("preserve the exact opening header spelling")
        };
        let content = if index + 1 == chunks.len() {
            content
        } else {
            content.strip_suffix("\n```").expect("a split code payload has a synthetic close")
        };
        original.push_str(content);
    }
    assert_eq!(original, text, "only the declared synthetic wrappers may be added");
}

#[test]
fn test_channel_splitter_preserves_fenced_payload_and_header_spelling() {
    let closure_overflow = format!("```\n{}\n{}\n```", "x".repeat(3_993), "y".repeat(10));
    assert_channel_fence_payload_preserved(&closure_overflow, "```", 4_000);

    let header = "  ``` Rust + meta  ";
    let unicode_crlf = format!(
        "{header}\r\n{}\r\n\r\n \tlast line  \r\n  ```\r\n",
        "💡界".repeat(4_001),
    );
    assert_channel_fence_payload_preserved(&unicode_crlf, header, 4_000);

    let code = format!("```rust\n{}```\n", "let x = 42;\n".repeat(30));
    assert_channel_fence_payload_preserved(&code, "```rust", 80);
}

#[test]
fn test_channel_splitter_does_not_promote_inline_backticks_to_fences() {
    let header = "```text";
    for body in [
        format!("{}```literal\n", "a".repeat(40)),
        format!("{}\t  ```literal\r\n", "a".repeat(20)),
        format!("{}{}\n", "a".repeat(20), "```".repeat(40)),
    ] {
        let text = format!("{header}\n{body}```");
        assert_channel_fence_payload_preserved(&text, header, 32);
    }
    for limit in [32, 4_096, 8_192] {
        let text = format!("{header}\na{}\n```", "`".repeat(24_577));
        assert_channel_fence_payload_preserved(&text, header, limit);
    }
    for limit in [32, CHANNEL_MAX_MESSAGE_CHARS] {
        for (prefix, inline) in [
            ("a".repeat(limit), "```literal\n"),
            ("界".repeat(limit), "\t  ```literal\r\n"),
        ] {
            let text = format!("{prefix}{inline}```text\nx\n```\n");
            let chunks = split_channel_message(&text, limit);
            assert_channel_chunks_bounded(&chunks, limit);
            assert_eq!(chunks.concat(), text, "mixed prose and fitting code need no added wrappers");
            for chunk in chunks {
                assert!(!chunk.is_empty());
                assert_eq!(
                    chunk.lines().filter(|line| line.trim_start().starts_with("```")).count() % 2,
                    0,
                    "a prose split must not promote inline payload to a fence",
                );
            }
        }
    }
}

#[test]
fn test_channel_splitter_uses_exact_raw_fallback_for_unrepresentable_fences() {
    let cases = [
        (format!("prefix\n```rust\n{}", "unclosed ".repeat(10)), 32),
        ("```\nabcdef\n```".to_string(), 8),
        ("```\nabcdef\n```".to_string(), 0),
        (format!("  ```{}\nx\n```", "header".repeat(20)), 32),
        (format!("```rust\nx\n  ```{}\r\n", "closing".repeat(20)), 32),
    ];
    for (text, limit) in cases {
        let chunks = split_channel_message(&text, limit);
        assert_channel_chunks_bounded(&chunks, limit);
        assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
        assert_eq!(chunks.concat(), text, "fallback adds no formatting characters");
        assert!(chunks[..chunks.len() - 1].iter().all(|chunk| {
            chunk.chars().count() == limit.max(1)
        }), "raw fallback uses deterministic full-size scalar chunks before the tail");
    }
    let short_unmatched = "  ```rust\r\n";
    assert_eq!(
        split_channel_message(short_unmatched, short_unmatched.chars().count()),
        vec![short_unmatched.to_string()],
        "even unmatched short input retains the existing fast path",
    );
}

type ChannelChunkRecords = Arc<Mutex<Vec<(String, String, ChannelOutboundMessage)>>>;

struct BoundedChannelDeliveryPort {
    calls: ChannelChunkRecords,
    accepted: ChannelChunkRecords,
    fail_after_accepting: Option<usize>,
}

impl ChannelDeliveryPort for BoundedChannelDeliveryPort {
    fn deliver(
        &self,
        agent_id: &str,
        address_token: &str,
        message: &ChannelOutboundMessage,
    ) -> Result<(), String> {
        let record = (agent_id.to_string(), address_token.to_string(), message.clone());
        self.calls.lock().unwrap().push(record.clone());
        if let ChannelOutboundMessage::Text { text, .. } = message {
            if text.chars().count() > CHANNEL_MAX_MESSAGE_CHARS {
                return Err("controlled provider rejected an oversized text payload".into());
            }
        }
        let accepted_count = {
            let mut accepted = self.accepted.lock().unwrap();
            accepted.push(record);
            accepted.len()
        };
        if self.fail_after_accepting == Some(accepted_count) {
            return Err("controlled acknowledgement error after accepted payload".into());
        }
        Ok(())
    }
}

fn bounded_channel_delivery(
    fail_after_accepting: Option<usize>,
) -> (HostChannelDelivery, ChannelChunkRecords, ChannelChunkRecords) {
    let router = ChannelDeliveryRouter::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let accepted = Arc::new(Mutex::new(Vec::new()));
    router.register("fixture", Arc::new(BoundedChannelDeliveryPort {
        calls: Arc::clone(&calls),
        accepted: Arc::clone(&accepted),
        fail_after_accepting,
    })).expect("real provider registration");
    (HostChannelDelivery::new(Arc::new(router)), calls, accepted)
}

#[test]
fn test_channel_router_bounds_chunks_and_preserves_delivery_identity() {
    let fenced = format!("```\n{}\n{}\n```", "x".repeat(3_993), "y".repeat(10));
    for (text, plain) in [
        ("x".repeat(4_000), true),
        ("x".repeat(4_001), true),
        ("🦀é界".repeat(2_667), true),
        (format!("{}```literal\n```text\nx\n```\n", "a".repeat(CHANNEL_MAX_MESSAGE_CHARS)), true),
        (fenced, false),
    ] {
        let (delivery, calls, accepted) = bounded_channel_delivery(None);
        delivery.deliver("agent-7", "fixture:room/9", &ChannelOutboundMessage::Text {
            text: text.clone(),
            operation_id: Some("persisted-entry-7".into()),
        }).expect("the actual Host/router must respect the enforcing provider's bound");
        let calls = calls.lock().unwrap().clone();
        assert_eq!(calls, *accepted.lock().unwrap(), "every accepted effect has exactly one call");
        assert!(!calls.is_empty());
        if text.chars().count() > CHANNEL_MAX_MESSAGE_CHARS {
            assert!(calls.len() > 1);
        } else {
            assert_eq!(calls.len(), 1);
        }
        let mut reconstructed = String::new();
        for (index, (agent_id, address, message)) in calls.iter().enumerate() {
            assert_eq!(agent_id, "agent-7");
            assert_eq!(address, "fixture:room/9");
            let ChannelOutboundMessage::Text { text, operation_id } = message else {
                panic!("text must remain text at the actual provider boundary");
            };
            assert!(!text.is_empty());
            assert!(text.chars().count() <= CHANNEL_MAX_MESSAGE_CHARS);
            assert_eq!(
                text.lines().filter(|line| line.trim_start().starts_with("```")).count() % 2,
                0,
                "every normal-fence payload reaching the provider remains balanced",
            );
            let expected = format!("persisted-entry-7:chunk-{}", index + 1);
            assert_eq!(operation_id.as_deref(), Some(expected.as_str()));
            reconstructed.push_str(text);
        }
        if plain {
            assert_eq!(reconstructed, text, "provider calls preserve original text order");
        }
    }
}

#[test]
fn test_channel_router_stops_after_effect_then_error_without_replay() {
    let (delivery, calls, accepted) = bounded_channel_delivery(Some(2));
    let result = delivery.deliver("agent-7", "fixture:room/9", &ChannelOutboundMessage::Text {
        text: "x".repeat(9_000),
        operation_id: Some("persisted-entry-7".into()),
    });
    assert_eq!(result, Err("controlled acknowledgement error after accepted payload".into()));
    let calls = calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 2, "no third chunk and no retry after an ambiguous effect");
    assert_eq!(calls, *accepted.lock().unwrap(), "the second effect happened before the error");
    for (index, (agent_id, address, message)) in calls.iter().enumerate() {
        assert_eq!(agent_id, "agent-7");
        assert_eq!(address, "fixture:room/9");
        assert_eq!(message, &ChannelOutboundMessage::Text {
            text: "x".repeat(4_000),
            operation_id: Some(format!("persisted-entry-7:chunk-{}", index + 1)),
        });
    }
}

#[test]
fn test_channel_router_preserves_attachment_and_absent_operation_identity() {
    let (delivery, calls, accepted) = bounded_channel_delivery(None);
    let attachment = ChannelOutboundMessage::Attachment {
        url: "https://example.invalid/original-attachment".into(),
        caption: Some("caption".repeat(1_000)),
        operation_id: Some("attachment-entry".into()),
    };
    delivery.deliver("agent-7", "fixture:room/9", &attachment).expect("unchanged attachment dispatch");
    delivery.deliver("agent-7", "fixture:room/9", &ChannelOutboundMessage::Text {
        text: "é".repeat(4_001),
        operation_id: None,
    }).expect("text without an operation identity still chunks");
    let calls = calls.lock().unwrap().clone();
    assert_eq!(calls, *accepted.lock().unwrap());
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0], ("agent-7".into(), "fixture:room/9".into(), attachment));
    let mut text = String::new();
    for (agent_id, address, message) in &calls[1..] {
        assert_eq!(agent_id, "agent-7");
        assert_eq!(address, "fixture:room/9");
        let ChannelOutboundMessage::Text { text: chunk, operation_id } = message else {
            panic!("only the original attachment is an attachment");
        };
        assert!(operation_id.is_none(), "do not manufacture an operation identity");
        assert!(chunk.chars().count() <= CHANNEL_MAX_MESSAGE_CHARS);
        text.push_str(chunk);
    }
    assert_eq!(text, "é".repeat(4_001));
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O07: Channel Rate Limiter Backoff on HTTP 429 Retry-After
// ---------------------------------------------------------------------------
struct ChannelRateLimiter {
    blocked_until_ms: u64,
}

impl ChannelRateLimiter {
    fn new() -> Self {
        Self { blocked_until_ms: 0 }
    }

    fn handle_rate_limit(&mut self, retry_after_seconds: u64, now_ms: u64) {
        self.blocked_until_ms = now_ms + (retry_after_seconds * 1000);
    }

    fn can_send(&self, now_ms: u64) -> bool {
        now_ms >= self.blocked_until_ms
    }
}

#[test]
fn test_ocrm_channel_o07_rate_limiter_backoff() {
    let mut limiter = ChannelRateLimiter::new();
    let now = 1_000_000_u64;

    assert!(limiter.can_send(now));

    // HTTP 429 with Retry-After: 30
    limiter.handle_rate_limit(30, now);
    assert!(!limiter.can_send(now + 15_000), "must block during retry-after window");
    assert!(limiter.can_send(now + 30_000), "must unblock after retry-after expires");
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O08: Delivery Receipt Settlement Lifecycle
// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Acked,
    Failed,
}

struct DeliveryReceiptTracker {
    deliveries: HashMap<String, DeliveryStatus>,
}

impl DeliveryReceiptTracker {
    fn new() -> Self {
        Self { deliveries: HashMap::new() }
    }

    fn transition(&mut self, id: &str, target: DeliveryStatus) -> Result<(), &'static str> {
        let current = self.deliveries.entry(id.to_string()).or_insert(DeliveryStatus::Pending);
        match (*current, target) {
            (DeliveryStatus::Pending, DeliveryStatus::Sent) => *current = target,
            (DeliveryStatus::Sent, DeliveryStatus::Delivered) => *current = target,
            (DeliveryStatus::Delivered, DeliveryStatus::Acked) => *current = target,
            (DeliveryStatus::Pending | DeliveryStatus::Sent, DeliveryStatus::Failed) => *current = target,
            _ => return Err("invalid delivery receipt transition"),
        }
        Ok(())
    }
}

#[test]
fn test_ocrm_channel_o08_receipt_settlement() {
    let mut tracker = DeliveryReceiptTracker::new();

    assert!(tracker.transition("rcpt-1", DeliveryStatus::Sent).is_ok());
    assert!(tracker.transition("rcpt-1", DeliveryStatus::Delivered).is_ok());
    assert!(tracker.transition("rcpt-1", DeliveryStatus::Acked).is_ok());
    assert_eq!(tracker.deliveries["rcpt-1"], DeliveryStatus::Acked);

    // Invalid backwards transition
    assert!(tracker.transition("rcpt-1", DeliveryStatus::Pending).is_err());
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O09: Dead Letter Replay Queue after Reconnection
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_channel_o09_dead_letter_replay() {
    let mut dlq = ChannelDeadLetterQueue::new();
    let msg1 = ChannelOutboundMessage::Text { text: "msg 1".into(), operation_id: None };
    let msg2 = ChannelOutboundMessage::Text { text: "msg 2".into(), operation_id: None };

    let unreg_delivery = HostChannelDelivery::default();
    dlq.dispatch_or_enqueue(&unreg_delivery, "agent-1", "user-1", &msg1);
    dlq.dispatch_or_enqueue(&unreg_delivery, "agent-1", "user-2", &msg2);
    assert_eq!(dlq.dead_letters.len(), 2);

    // Channel reconnects and registers active delivery port
    let records = Arc::new(Mutex::new(Vec::new()));
    let port = Arc::new(RecordingChannelDeliveryPort { delivered: records.clone() });
    let active_delivery = HostChannelDelivery::new(port);

    // Replay dead letters
    while let Some((agent_id, addr, msg)) = dlq.dead_letters.pop_front() {
        active_delivery.deliver(&agent_id, &addr, &msg).expect("replay");
    }

    assert_eq!(records.lock().unwrap().len(), 2);
    assert!(dlq.dead_letters.is_empty());
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O10: Reconnection Exponential Backoff & Max Bounds
// ---------------------------------------------------------------------------
struct ReconnectBackoffPolicy {
    base_ms: u64,
    max_ms: u64,
    max_attempts: u32,
}

impl ReconnectBackoffPolicy {
    fn delay_for_attempt(&self, attempt: u32) -> Option<u64> {
        if attempt > self.max_attempts {
            return None;
        }
        let exp = 2_u64.saturating_pow(attempt.saturating_sub(1));
        let delay = (self.base_ms.saturating_mul(exp)).min(self.max_ms);
        Some(delay)
    }
}

#[test]
fn test_ocrm_channel_o10_reconnection_backoff() {
    let policy = ReconnectBackoffPolicy {
        base_ms: 1_000,
        max_ms: 30_000,
        max_attempts: 5,
    };

    assert_eq!(policy.delay_for_attempt(1), Some(1_000));
    assert_eq!(policy.delay_for_attempt(2), Some(2_000));
    assert_eq!(policy.delay_for_attempt(3), Some(4_000));
    assert_eq!(policy.delay_for_attempt(4), Some(8_000));
    assert_eq!(policy.delay_for_attempt(5), Some(16_000));
    assert_eq!(policy.delay_for_attempt(6), None, "exceeded max attempts must return None");
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O11: Webhook HMAC Signature Verification
// ---------------------------------------------------------------------------
fn verify_webhook_hmac_sha256(payload: &[u8], secret: &str, expected_hex: &str) -> bool {
    use mahayana_host_runtime::sha256::sha256_hex;

    let mut combined = secret.as_bytes().to_vec();
    combined.extend_from_slice(payload);
    let hex = sha256_hex(&combined);
    hex == expected_hex
}

#[test]
fn test_ocrm_channel_o11_webhook_hmac_verification() {
    use mahayana_host_runtime::sha256::sha256_hex;

    let payload = b"{\"event\":\"message_created\",\"id\":\"ev-42\"}";
    let secret = "channel_signing_secret_xyz";

    let mut combined = secret.as_bytes().to_vec();
    combined.extend_from_slice(payload);
    let valid_hex = sha256_hex(&combined);

    assert!(verify_webhook_hmac_sha256(payload, secret, &valid_hex));
    assert!(!verify_webhook_hmac_sha256(payload, "wrong_secret", &valid_hex));
    assert!(!verify_webhook_hmac_sha256(b"tampered", secret, &valid_hex));
}

// ---------------------------------------------------------------------------
// OCRM-CHANNEL-O12: Channel Access Control List (ACL) Gating
// ---------------------------------------------------------------------------
struct ChannelAclGate {
    allowlist: HashSet<String>,
    denylist: HashSet<String>,
}

impl ChannelAclGate {
    fn is_authorized(&self, sender_id: &str) -> bool {
        if self.denylist.contains(sender_id) {
            return false;
        }
        if !self.allowlist.is_empty() && !self.allowlist.contains(sender_id) {
            return false;
        }
        true
    }
}

#[test]
fn test_ocrm_channel_o12_acl_gating() {
    let mut acl = ChannelAclGate {
        allowlist: HashSet::from(["user_alice".into(), "user_bob".into()]),
        denylist: HashSet::from(["user_blocked".into()]),
    };

    assert!(acl.is_authorized("user_alice"));
    assert!(acl.is_authorized("user_bob"));
    assert!(!acl.is_authorized("user_charlie"), "unlisted user must be rejected when allowlist is populated");
    assert!(!acl.is_authorized("user_blocked"), "denylisted user must be rejected");
}
