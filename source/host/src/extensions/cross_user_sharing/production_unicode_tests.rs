//! MRU-v1 exercises the existing production callback and ordered publisher.
//! The relay is controlled; attachment resolution uses the actual Rust owners.

use super::*;
use crate::extensions::attachments::attachments_service::AttachmentsBox;
use crate::extensions::attachments::generate_image_service::GenerateImageAuth;
use crate::extensions::cross_user_sharing::xuser_relay::SandXuserRelayHttpError;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::transcript::transcript_manager::TranscriptManager;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

const IMAGE_BASE64: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAusB9WFhvokAAAAASUVORK5CYII=";
const IMAGE_FILENAME: &str = "报告-é-🦀.PNG";

struct MruScratch(PathBuf);

impl MruScratch {
    fn new() -> Arc<Self> {
        let path = std::env::temp_dir().join(format!(
            "fabushi-mru-production-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4(),
        ));
        fs::create_dir_all(&path).expect("create owned MRU fixture");
        Arc::new(Self(path))
    }
}

impl Drop for MruScratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct MruUnusedPorts {
    // The actual service, and therefore an active publisher callback, retains
    // the fixture directory until all its owners have released it.
    _scratch: Arc<MruScratch>,
}

impl GenerateImageAuth for MruUnusedPorts {
    fn get_access_token(&self) -> Result<String, String> {
        panic!("local attachment resolution must not ask for image credentials")
    }

    fn get_machine_id(&self) -> Result<String, String> {
        panic!("local attachment resolution must not ask for a machine identity")
    }
}

impl AttachmentsBox for MruUnusedPorts {
    fn run_state(&self, _agent_id: &str) -> Result<String, String> {
        panic!("local attachment resolution must not inspect a Box")
    }

    fn upload_file(&self, _agent_id: &str, _path: &str, _data: &[u8]) -> Result<(), String> {
        panic!("local attachment resolution must not upload to a Box")
    }
}

#[derive(Default)]
struct MruRelay {
    sent: Mutex<Vec<Value>>,
}

impl XuserRelayTransport for MruRelay {
    fn request(
        &self,
        method: &str,
        url: &str,
        token: &str,
        body: Option<&Value>,
    ) -> Result<Value, SandXuserRelayHttpError> {
        assert_eq!(method, "POST");
        assert_eq!(url, "https://mru-relay.invalid/sand/xuser/send");
        assert_eq!(token, "mru-controlled-token");
        self.sent
            .lock()
            .expect("relay observations")
            .push(body.expect("room publication payload").clone());
        // No server timestamp: restamping and a real backend are outside MRU.
        Ok(json!({}))
    }
}

struct MruFixture {
    scratch: Arc<MruScratch>,
    host: Arc<ProductionXuserHost>,
    relay: Arc<MruRelay>,
    _service: Arc<SandXuserSharingService>,
    image_path: PathBuf,
    image_url: String,
    image_bytes: Vec<u8>,
}

impl MruFixture {
    fn new() -> Self {
        let scratch = MruScratch::new();
        let image_path = scratch.0.join(IMAGE_FILENAME);
        let image_bytes = STANDARD.decode(IMAGE_BASE64).expect("literal PNG fixture");
        fs::write(&image_path, &image_bytes).expect("write owned Unicode image");
        let image_url = url::Url::from_file_path(&image_path)
            .expect("absolute fixture file URL")
            .to_string();
        let relay = Arc::new(MruRelay::default());
        let client = Arc::new(SandXuserRelayClient::new(
            "https://mru-relay.invalid".into(),
            Arc::new(|| Ok("mru-controlled-token".into())),
            relay.clone(),
        ));
        let service = Arc::new(SandXuserSharingService::new(Arc::clone(&client)));
        service.set_enabled(true);
        service.set_self_auth_id(Some("mru-self".into()));
        let sessions = Arc::new(ProductionSessionWorkers::with_agents_root(
            scratch.0.join("agents"),
            500,
        ));
        let manager = TranscriptManager::new(&scratch.0, sessions);
        let ports = Arc::new(MruUnusedPorts {
            _scratch: Arc::clone(&scratch),
        });
        let attachments = Arc::new(AttachmentsService::new(
            scratch.0.clone(),
            ports.clone(),
            ports,
            None,
        ));
        let host = Arc::new(ProductionXuserHost {
            relay: client,
            service: Arc::downgrade(&service),
            shared_rooms: manager.shared_rooms(),
            attachments,
            run_remote_requested_turn: Arc::new(|_, _, _| {
                panic!("attachment publication must not start a remote turn")
            }),
            run_shared_room_turn: Arc::new(|_, _| {
                panic!("attachment publication must not start a group turn")
            }),
            remote_turns: Arc::new(Mutex::new(Weak::new())),
            departures: SandXuserDepartureObligations::new(
                Arc::new(SandXuserPendingDepartureStore::new(&scratch.0)),
                Arc::new(SandXuserRoomTombstoneStore::new(&scratch.0)),
            ),
        });
        Self {
            scratch,
            host,
            relay,
            _service: service,
            image_path,
            image_url,
            image_bytes,
        }
    }

    fn missing_single_slash_file_url(&self) -> String {
        // A fresh root-level name is never written by this fixture. The é
        // crosses byte 7, independently of the runner's temp-directory path.
        format!("file:/é-mru-{}/missing.png", uuid::Uuid::new_v4())
    }
}

#[test]
fn mru_production_attachment_callback_preserves_unicode_and_refusal_controls() {
    let fixture = MruFixture::new();
    let expected_upload = Some(ResolvedChannelAttachment::Upload {
        is_image: true,
        bytes: fixture.image_bytes.clone(),
        filename: IMAGE_FILENAME.into(),
        mime: "image/png".into(),
    });
    for source in [
        fixture.image_path.to_str().expect("Unicode fixture path"),
        fixture.image_url.as_str(),
    ] {
        assert_eq!(
            fixture.host.attachments.resolve_channel_attachment(Some(source)),
            expected_upload,
            "the actual attachment owner must preserve filename, MIME and bytes",
        );
        assert_eq!(
            XuserEntryPublisherHost::resolve_attachment(fixture.host.as_ref(), source)
                .expect("production attachment callback"),
            Some(ResolvedXuserAttachment {
                data: fixture.image_bytes.clone(),
                mime_type: "image/png".into(),
            }),
        );
    }

    let missing = fixture.missing_single_slash_file_url();
    for source in [
        missing.as_str(),
        "file://é.invalid/a.png",
        "文档/图片.png",
        "https://mru-remote.invalid/image.png",
        "data:image/png;base64,AQID",
        "ftp://mru-remote.invalid/image.png",
    ] {
        assert_eq!(
            XuserEntryPublisherHost::resolve_attachment(fixture.host.as_ref(), source)
                .expect("unresolvable references are ordinary callback results"),
            None,
            "reference must be omitted without local/remote authority expansion: {source}",
        );
    }
    assert!(fixture.relay.sent.lock().expect("relay observations").is_empty());
    assert_eq!(fs::read(&fixture.image_path).unwrap(), fixture.image_bytes);

    let ports = Arc::new(MruUnusedPorts {
        _scratch: Arc::clone(&fixture.scratch),
    });
    let reopened = AttachmentsService::new(
        fixture.scratch.0.clone(),
        ports.clone(),
        ports,
        None,
    );
    assert_eq!(
        reopened.resolve_channel_attachment(Some(&fixture.image_url)),
        expected_upload,
        "a fresh service resolves the same unchanged stored image",
    );
}

#[test]
fn mru_ordered_room_publication_survives_unicode_attachment_refusals() {
    let fixture = MruFixture::new();
    let publisher = SandXuserEntryPublisher::new(fixture.host.clone());
    let missing = fixture.missing_single_slash_file_url();
    let mut expected = Vec::new();
    for (id, source, text, stamp) in [
        ("unicode-path", missing.as_str(), "保留第一条文字", 1001),
        ("unicode-authority", "file://é.invalid/a.png", "保留第二条文字", 1002),
    ] {
        publisher
            .publish_entry_ordered_and_wait(
                "mru-room",
                &json!({
                    "kind": "message", "id": id, "role": "user",
                    "content": text, "timestampMs": stamp,
                    "images": [{"url": source, "alt": "must be omitted"}],
                }),
            )
            .expect("Unicode refusal must not terminate the room worker");
        expected.push(json!({
            "kind": "room-entry", "roomId": "mru-room",
            "entry": {
                "kind": "human-message", "entryId": id,
                "authorAuthId": "mru-self", "authorName": "Host",
                "text": text, "images": [], "timestampMs": stamp as f64,
            },
        }));
    }

    publisher
        .publish_entry_ordered_and_wait(
            "mru-room",
            &json!({
                "kind": "message", "id": "unicode-image", "role": "user",
                "content": "图片继续发布", "timestampMs": 1003,
                "clientNonce": "mru-nonce",
                "images": [{"url": fixture.image_url, "alt": "说明-é-🦀"}],
            }),
        )
        .expect("same room worker must publish a real local image after refusals");
    expected.push(json!({
        "kind": "room-entry", "roomId": "mru-room",
        "entry": {
            "kind": "human-message", "entryId": "unicode-image",
            "authorAuthId": "mru-self", "authorName": "Host",
            "text": "图片继续发布", "timestampMs": 1003.0,
            "clientNonce": "mru-nonce",
            "images": [{
                "base64": IMAGE_BASE64, "mediaType": "image/png", "alt": "说明-é-🦀",
            }],
        },
    }));

    publisher
        .publish_entry_ordered_and_wait(
            "mru-room",
            &json!({
                "kind": "send-message", "id": "agent-after-image",
                "author": {"id": "mru-agent", "name": "Mru Agent"},
                "message": {"type": "text", "content": "later agent reply"},
                "timestampMs": 1004,
            }),
        )
        .expect("later text publication must retain room ordering");
    expected.push(json!({
        "kind": "room-entry", "roomId": "mru-room",
        "entry": {
            "kind": "agent-message", "entryId": "agent-after-image",
            "agentOwnerAuthId": "mru-self", "agentId": "mru-agent",
            "authorName": "Mru Agent", "text": "later agent reply",
            "images": [], "timestampMs": 1004.0,
        },
    }));
    publisher.flush_room("mru-room").expect("room worker remains alive");
    assert_eq!(*fixture.relay.sent.lock().expect("relay observations"), expected);
    assert_eq!(fs::read(&fixture.image_path).unwrap(), fixture.image_bytes);
    drop(publisher);
}
