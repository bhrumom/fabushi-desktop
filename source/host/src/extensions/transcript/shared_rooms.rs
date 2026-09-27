use base64::Engine as _;

pub fn decode_avatar_data_url(data_url: &str) -> Option<Vec<u8>> {
    const MARKER: &str = ";base64,";
    if !data_url.starts_with("data:image/") {
        return None;
    }
    let index = data_url.find(MARKER)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data_url[index + MARKER.len()..])
        .ok()?;
    (!bytes.is_empty()).then_some(bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomEntryKind {
    Message,
    SendMessage,
    Other,
}

pub fn is_room_content_entry(kind: RoomEntryKind) -> bool {
    matches!(kind, RoomEntryKind::Message | RoomEntryKind::SendMessage)
}

pub fn normalize_group_message(text: &str, max_chars: usize) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "(pass)" {
        return None;
    }
    Some(trimmed.chars().take(max_chars).collect())
}

pub fn can_post_to_group(member_ids: &[String], from_agent_id: &str) -> bool {
    member_ids.iter().any(|id| id == from_agent_id)
}
