use fabushi_messaging_core::*;
use std::collections::{BTreeMap, BTreeSet};

fn context(actor_id: &str, request_id: &str) -> RequestContext {
    RequestContext {
        request_id: request_id.into(),
        device_id: "desktop:story-contract".into(),
        actor_id: ActorId::new(actor_id),
        session_id: "story-contract".into(),
        sent_at_ms: 10,
    }
}

fn profile(service: &mut MessagingService<MemoryStateStore>, actor_id: &str, name: &str) {
    service
        .handle(
            ClientEnvelope::new(
                context(actor_id, &format!("profile:{actor_id}")),
                ClientCommand::UpsertProfile {
                    actor: Actor::human(actor_id, name),
                },
            ),
            10,
        )
        .unwrap();
}

fn story(owner_id: &str) -> Story {
    Story {
        id: StoryId("story:contract:1".into()),
        owner_id: ActorId::new(owner_id),
        media: MediaRef {
            id: "media:story:1".into(),
            file_name: Some("story.jpg".into()),
            mime_type: Some("image/jpeg".into()),
            size_bytes: Some(1024),
            width: Some(1080),
            height: Some(1920),
            duration_ms: None,
            thumbnail_id: None,
            local_path: None,
            remote_url: Some("https://media.fabushi.invalid/story.jpg".into()),
            content_hash: None,
        },
        caption: FormattedText::plain("Story contract"),
        privacy: StoryPrivacy {
            kind: StoryPrivacyKind::Everyone,
            included_actor_ids: BTreeSet::new(),
            excluded_actor_ids: BTreeSet::new(),
        },
        created_at_ms: 10,
        expires_at_ms: 10_000,
        edited_at_ms: None,
        pinned_to_profile: false,
        protected_content: false,
        allow_replies: true,
        views: BTreeMap::new(),
    }
}

#[test]
fn list_stories_projects_visible_story_without_sync_side_effects_and_view_mutates_canonical_state() {
    let mut service = MessagingService::load(MemoryStateStore::default()).unwrap();
    profile(&mut service, "human:owner", "Owner");
    profile(&mut service, "human:viewer", "Viewer");

    service
        .handle(
            ClientEnvelope::new(
                context("human:owner", "publish:1"),
                ClientCommand::PublishStory {
                    story: story("human:owner"),
                },
            ),
            20,
        )
        .unwrap();

    let cursor_before_list = service.cursor();
    let listed = service
        .handle(
            ClientEnvelope::new(
                context("human:viewer", "list:1"),
                ClientCommand::ListStories { limit: 20 },
            ),
            30,
        )
        .unwrap();

    assert_eq!(service.cursor(), cursor_before_list);
    assert!(matches!(
        listed.as_slice(),
        [ServerEnvelope {
            event: ServerEvent::StoriesSnapshot { stories },
            ..
        }] if stories.len() == 1 && stories[0].id == StoryId("story:contract:1".into())
    ));

    let viewed = service
        .handle(
            ClientEnvelope::new(
                context("human:viewer", "view:1"),
                ClientCommand::ViewStory {
                    story_id: StoryId("story:contract:1".into()),
                },
            ),
            40,
        )
        .unwrap();

    assert!(matches!(
        viewed.as_slice(),
        [ServerEnvelope {
            event: ServerEvent::StoryChanged { story },
            ..
        }] if story.views.contains_key(&ActorId::new("human:viewer"))
    ));
}
