use std::fs;
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::inference::provider_session::{
    RoutedProvider, configured_routed_provider, decode_sse_stream,
};
use serde_json::json;

#[test]
fn provider_session_reads_router_settings_and_streams_sse_events() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fabushi-provider-settings-{}-{suffix}.json",
        std::process::id()
    ));
    fs::write(&path, r#"{"router":{"provider":"codex"}}"#)
        .expect("write settings");
    assert_eq!(
        configured_routed_provider(&path),
        Some(RoutedProvider::Codex)
    );
    fs::remove_file(path).expect("remove settings");

    let stream = b"event: ignored\r\ndata: {\"type\":\"response.output_text.delta\",\"delta\":\"hi\"}\r\n\r\ndata: [DONE]\r\n\r\n";
    let mut events = Vec::new();
    decode_sse_stream(Cursor::new(stream.as_slice()), |event| {
        events.push(event);
        Ok(())
    })
    .expect("decode SSE");
    assert_eq!(
        events,
        vec![json!({
            "type":"response.output_text.delta",
            "delta":"hi"
        })]
    );
}
