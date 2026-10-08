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
        anonymous_view_count: 0,
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

#[test]
fn story_stealth_requires_entitlement_fences_duplicates_persists_and_anonymizes_views() {
    let mut service = MessagingService::load(MemoryStateStore::default()).unwrap();
    profile(&mut service, "human:owner", "Owner");
    profile(&mut service, "human:viewer", "Viewer");

    service
        .handle(
            ClientEnvelope::new(
                context("human:owner", "publish:stealth"),
                ClientCommand::PublishStory {
                    story: story("human:owner"),
                },
            ),
            100,
        )
        .unwrap();

    service
        .handle(
            ClientEnvelope::new(
                context("human:viewer", "view:before"),
                ClientCommand::ViewStory {
                    story_id: StoryId("story:contract:1".into()),
                },
            ),
            200,
        )
        .unwrap();

    let denied = service
        .handle(
            ClientEnvelope::new(
                context("human:viewer", "stealth:denied"),
                ClientCommand::ActivateStoryStealth {
                    request_id: "activation:1".into(),
                },
            ),
            250,
        )
        .unwrap_err();
    assert!(matches!(
        denied,
        MessagingServiceError::Engine(EngineError::StoryStealthEntitlementRequired)
    ));

    service
        .reconcile_entitlement(
            Entitlement {
                id: "entitlement:stealth:viewer".into(),
                owner_id: ActorId::new("human:viewer"),
                product_id: STORY_STEALTH_PRODUCT_ID.into(),
                order_id: "order:stealth".into(),
                starts_at_ms: 0,
                expires_at_ms: None,
                revoked_at_ms: None,
            },
            260,
        )
        .unwrap();

    let activated = service
        .handle(
            ClientEnvelope::new(
                context("human:viewer", "stealth:activate"),
                ClientCommand::ActivateStoryStealth {
                    request_id: "activation:1".into(),
                },
            ),
            300,
        )
        .unwrap();
    assert!(activated.iter().any(|envelope| matches!(
        &envelope.event,
        ServerEvent::StoryStealthChanged { state }
            if state.enabled_till_ms == 300 + STORY_STEALTH_ACTIVE_MS
                && state.cooldown_till_ms == 300 + STORY_STEALTH_COOLDOWN_MS
    )));
    assert!(!service
        .engine()
        .state()
        .stories
        .get(&StoryId("story:contract:1".into()))
        .unwrap()
        .views
        .contains_key(&ActorId::new("human:viewer")));

    let cursor_before_duplicate = service.cursor();
    service
        .handle(
            ClientEnvelope::new(
                context("human:viewer", "stealth:duplicate"),
                ClientCommand::ActivateStoryStealth {
                    request_id: "activation:1".into(),
                },
            ),
            350,
        )
        .unwrap();
    assert_eq!(service.cursor(), cursor_before_duplicate + 1);
    let state_after_duplicate = service
        .engine()
        .state()
        .story_stealth
        .get(&ActorId::new("human:viewer"))
        .unwrap()
        .clone();
    assert_eq!(state_after_duplicate.enabled_till_ms, 300 + STORY_STEALTH_ACTIVE_MS);

    service
        .handle(
            ClientEnvelope::new(
                context("human:viewer", "view:anonymous"),
                ClientCommand::ViewStory {
                    story_id: StoryId("story:contract:1".into()),
                },
            ),
            400,
        )
        .unwrap();
    let canonical = service
        .engine()
        .state()
        .stories
        .get(&StoryId("story:contract:1".into()))
        .unwrap();
    assert!(!canonical.views.contains_key(&ActorId::new("human:viewer")));
    assert!(canonical.anonymous_view_count >= 2);

    let store = service.into_store();
    let mut restored = MessagingService::load(store).unwrap();
    let status = restored
        .handle(
            ClientEnvelope::new(
                context("human:viewer", "stealth:status"),
                ClientCommand::StoryStealthStatus,
            ),
            450,
        )
        .unwrap();
    assert!(matches!(
        status.as_slice(),
        [ServerEnvelope {
            event: ServerEvent::StoryStealthStatus { state, entitled: true },
            ..
        }] if state.enabled_till_ms == 300 + STORY_STEALTH_ACTIVE_MS
    ));

    let cooldown = restored
        .handle(
            ClientEnvelope::new(
                context("human:viewer", "stealth:cooldown"),
                ClientCommand::ActivateStoryStealth {
                    request_id: "activation:2".into(),
                },
            ),
            300 + STORY_STEALTH_ACTIVE_MS + 1,
        )
        .unwrap_err();
    assert!(matches!(
        cooldown,
        MessagingServiceError::Engine(EngineError::StoryStealthCooldown { .. })
    ));
}
