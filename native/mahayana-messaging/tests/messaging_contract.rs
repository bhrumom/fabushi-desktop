use fabushi_messaging_core::*;

fn participant(actor_id: &str, role: ParticipantRole) -> Participant {
    Participant {
        actor_id: ActorId::new(actor_id),
        role,
        joined_at_ms: 1,
        muted_until_ms: None,
    }
}

#[test]
fn humans_and_bots_share_the_same_conversation_engine() {
    let mut engine = MessagingEngine::new();
    engine
        .execute(Command::UpsertActor {
            actor: Actor::human("human:1", "善友"),
        })
        .unwrap();
    engine
        .execute(Command::UpsertActor {
            actor: Actor::assistant("agent:mahayana", "大乘助手"),
        })
        .unwrap();
    engine
        .execute(Command::UpsertActor {
            actor: Actor::bot("bot:publisher", "发布机器人"),
        })
        .unwrap();

    let conversation = Conversation::direct(
        "chat:unified",
        "善友 · 大乘助手 · 发布机器人",
        vec![
            participant("human:1", ParticipantRole::Owner),
            participant("agent:mahayana", ParticipantRole::Member),
            participant("bot:publisher", ParticipantRole::Member),
        ],
        10,
    );
    engine
        .execute(Command::UpsertConversation { conversation })
        .unwrap();
    let events = engine
        .execute(Command::QueueMessage {
            conversation_id: ConversationId::new("chat:unified"),
            local_message_id: MessageId::new("local:1"),
            client_message_id: ClientMessageId("client:1".into()),
            sender_id: ActorId::new("human:1"),
            content: MessageContent::Text {
                text: FormattedText::plain("请两个机器人协作处理"),
            },
            reply_to_message_id: None,
            thread_root_message_id: None,
            created_at_ms: 20,
            scheduled_at_ms: None,
            silent: false,
            protected_content: false,
        })
        .unwrap();
    assert!(matches!(events.as_slice(), [Event::MessageQueued { .. }]));
    assert_eq!(engine.state().actors.len(), 3);
    assert_eq!(
        engine.state().messages[&ConversationId::new("chat:unified")].len(),
        1
    );
}

#[test]
fn invoice_and_paid_order_use_the_same_chat_domain() {
    let mut engine = MessagingEngine::new();
    engine
        .execute(Command::UpsertActor {
            actor: Actor::human("buyer", "购买者"),
        })
        .unwrap();
    engine
        .execute(Command::UpsertActor {
            actor: Actor::bot("seller", "商店机器人"),
        })
        .unwrap();
    engine
        .execute(Command::UpsertConversation {
            conversation: Conversation::direct(
                "chat:shop",
                "商店",
                vec![
                    participant("buyer", ParticipantRole::Owner),
                    participant("seller", ParticipantRole::Member),
                ],
                1,
            ),
        })
        .unwrap();
    let invoice = Invoice {
        id: "invoice:1".into(),
        conversation_id: ConversationId::new("chat:shop"),
        seller_id: ActorId::new("seller"),
        title: "Mini App Pro".into(),
        description: "数字商品".into(),
        kind: InvoiceKind::DigitalGoods,
        currency: "USD".into(),
        prices: vec![PriceLine {
            label: "Pro".into(),
            amount: Money::new("USD", 999),
        }],
        payload: "product=pro".into(),
        provider_id: "fabushi-pay".into(),
        start_parameter: None,
        request_name: false,
        request_email: true,
        request_phone: false,
        request_shipping_address: false,
        flexible_shipping: false,
        created_at_ms: 2,
        expires_at_ms: None,
    };
    engine.execute(Command::CreateInvoice { invoice }).unwrap();
    engine
        .execute(Command::UpsertOrder {
            order: PaymentOrder {
                id: "order:1".into(),
                invoice_id: "invoice:1".into(),
                buyer_id: ActorId::new("buyer"),
                status: PaymentStatus::Pending,
                amount: Money::new("USD", 999),
                customer: None,
                provider_payment_id: None,
                provider_receipt_url: None,
                created_at_ms: 3,
                updated_at_ms: 3,
            },
        })
        .unwrap();
    engine.complete_paid_order("order:1", 4).unwrap();
    assert_eq!(engine.state().orders["order:1"].status, PaymentStatus::Paid);
}

#[test]
fn mini_app_permissions_are_enforced_before_host_calls() {
    let mut engine = MessagingEngine::new();
    engine
        .execute(Command::UpsertActor {
            actor: Actor::human("human:1", "善友"),
        })
        .unwrap();
    let manifest = MiniAppManifest {
        id: "mini:shop".into(),
        name: "商店".into(),
        version: "1.0.0".into(),
        description: "支付测试".into(),
        icon_url: None,
        start_url: "https://mini.example.invalid".into(),
        allowed_origins: vec!["https://mini.example.invalid".into()],
        requested_permissions: vec![MiniAppPermission::Payments],
        bot_actor_id: None,
        verified: true,
    };
    engine
        .execute(Command::InstallMiniApp { manifest })
        .unwrap();
    engine
        .execute(Command::GrantMiniApp {
            grant: MiniAppGrant {
                mini_app_id: "mini:shop".into(),
                actor_id: ActorId::new("human:1"),
                permissions: vec![MiniAppPermission::Payments],
                granted_at_ms: 1,
                expires_at_ms: None,
            },
        })
        .unwrap();
    engine
        .execute(Command::OpenMiniApp {
            session: MiniAppSession {
                id: "session:1".into(),
                mini_app_id: "mini:shop".into(),
                actor_id: ActorId::new("human:1"),
                conversation_id: None,
                start_parameter: None,
                granted_permissions: vec![MiniAppPermission::Payments],
                opened_at_ms: 2,
                expires_at_ms: 20,
            },
        })
        .unwrap();
    let error = engine
        .execute(Command::MiniAppCall {
            session_id: "session:1".into(),
            request_id: "request:1".into(),
            request: MiniAppRequest::RequestLocation,
        })
        .unwrap_err();
    assert_eq!(
        error,
        EngineError::MiniAppPermissionDenied(MiniAppPermission::Location)
    );
}

#[test]
fn wire_protocol_is_fabushi_owned_and_versioned() {
    let envelope = ClientEnvelope::new(
        RequestContext {
            request_id: "req:1".into(),
            device_id: "desktop:1".into(),
            actor_id: ActorId::new("human:1"),
            session_id: "session:1".into(),
            sent_at_ms: 10,
        },
        ClientCommand::Sync {
            cursor: None,
            limit: 100,
        },
    );
    let json = serde_json::to_string(&envelope).unwrap();
    assert!(json.contains("protocolVersion"));
    assert!(!json.to_ascii_lowercase().contains("mtproto"));
    assert_eq!(
        envelope.protocol_version,
        FABUSHI_MESSAGING_PROTOCOL_VERSION
    );
}

#[test]
fn formatted_text_prefix_shifts_entities_by_utf16_units_atomically() {
    let mut text = FormattedText {
        text: "善友 bold".into(),
        entities: vec![TextEntity {
            offset_utf16: 3,
            length_utf16: 4,
            kind: TextEntityKind::Bold,
        }],
    };
    let prefix = "🔗 https://fabushi.example\n";
    let shift = u32::try_from(prefix.encode_utf16().count()).unwrap();

    text.prepend_plain_text(prefix).unwrap();

    assert_eq!(text.text, format!("{prefix}善友 bold"));
    assert_eq!(text.entities[0].offset_utf16, 3 + shift);
    assert_eq!(text.entities[0].length_utf16, 4);

    let mut overflow = FormattedText {
        text: "x".into(),
        entities: vec![TextEntity {
            offset_utf16: u32::MAX,
            length_utf16: 1,
            kind: TextEntityKind::Bold,
        }],
    };
    let before = overflow.clone();
    assert_eq!(
        overflow.prepend_plain_text("x"),
        Err(TextEntityOffsetOverflow)
    );
    assert_eq!(overflow, before);
}

#[test]
fn forwarding_preserves_origin_and_rejects_protected_content() {
    let mut engine = MessagingEngine::new();
    engine
        .execute(Command::UpsertActor {
            actor: Actor::human("human:forwarder", "转发者"),
        })
        .unwrap();
    for (id, title) in [("chat:source", "来源"), ("chat:destination", "目标")] {
        engine
            .execute(Command::UpsertConversation {
                conversation: Conversation::direct(
                    id,
                    title,
                    vec![participant("human:forwarder", ParticipantRole::Owner)],
                    1,
                ),
            })
            .unwrap();
    }
    engine
        .execute(Command::QueueMessage {
            conversation_id: ConversationId::new("chat:source"),
            local_message_id: MessageId::new("source:1"),
            client_message_id: ClientMessageId("client:source".into()),
            sender_id: ActorId::new("human:forwarder"),
            content: MessageContent::Text {
                text: FormattedText::plain("可转发内容"),
            },
            reply_to_message_id: None,
            thread_root_message_id: None,
            created_at_ms: 2,
            scheduled_at_ms: None,
            silent: false,
            protected_content: false,
        })
        .unwrap();
    engine
        .execute(Command::ForwardMessage {
            source_conversation_id: ConversationId::new("chat:source"),
            message_id: MessageId::new("source:1"),
            destination_conversation_id: ConversationId::new("chat:destination"),
            local_message_id: MessageId::new("forward:1"),
            client_message_id: ClientMessageId("client:forward".into()),
            sender_id: ActorId::new("human:forwarder"),
            thread_root_message_id: Some(MessageId::new("topic:destination")),
            created_at_ms: 3,
            scheduled_at_ms: Some(30),
            silent: true,
            privacy: ForwardPrivacy::default(),
        })
        .unwrap();
    let forwarded = &engine.state().messages[&ConversationId::new("chat:destination")]
        [&MessageId::new("forward:1")];
    assert_eq!(
        forwarded.forward_origin.as_deref(),
        Some("chat:source:source:1")
    );
    assert!(matches!(forwarded.content, MessageContent::Text { .. }));
    assert_eq!(
        forwarded.thread_root_message_id.as_ref(),
        Some(&MessageId::new("topic:destination"))
    );
    assert_eq!(forwarded.scheduled_at_ms, Some(30));
    assert!(forwarded.silent);

    engine
        .execute(Command::QueueMessage {
            conversation_id: ConversationId::new("chat:source"),
            local_message_id: MessageId::new("source:captioned"),
            client_message_id: ClientMessageId("client:captioned".into()),
            sender_id: ActorId::new("human:forwarder"),
            content: MessageContent::Photo {
                media: MediaRef {
                    id: "media:captioned".into(),
                    file_name: Some("photo.jpg".into()),
                    mime_type: Some("image/jpeg".into()),
                    size_bytes: Some(1),
                    width: Some(1),
                    height: Some(1),
                    duration_ms: None,
                    thumbnail_id: None,
                    local_path: None,
                    remote_url: None,
                    content_hash: None,
                },
                caption: FormattedText::plain("private caption"),
                spoiler: false,
            },
            reply_to_message_id: None,
            thread_root_message_id: None,
            created_at_ms: 3,
            scheduled_at_ms: None,
            silent: false,
            protected_content: false,
        })
        .unwrap();
    engine
        .execute(Command::ForwardMessage {
            source_conversation_id: ConversationId::new("chat:source"),
            message_id: MessageId::new("source:captioned"),
            destination_conversation_id: ConversationId::new("chat:destination"),
            local_message_id: MessageId::new("forward:privacy"),
            client_message_id: ClientMessageId("client:forward-privacy".into()),
            sender_id: ActorId::new("human:forwarder"),
            thread_root_message_id: None,
            created_at_ms: 4,
            scheduled_at_ms: None,
            silent: false,
            privacy: ForwardPrivacy {
                drop_sender_names: false,
                drop_captions: true,
            },
        })
        .unwrap();
    let private_forward = &engine.state().messages[&ConversationId::new("chat:destination")]
        [&MessageId::new("forward:privacy")];
    assert_eq!(private_forward.forward_origin, None);
    match &private_forward.content {
        MessageContent::Photo { caption, .. } => {
            assert!(caption.text.is_empty());
            assert!(caption.entities.is_empty());
        }
        other => panic!("expected photo, got {other:?}"),
    }

    let mut blocked_destination = Conversation::direct(
        "chat:blocked-forward",
        "Blocked forward",
        vec![participant("human:forwarder", ParticipantRole::Owner)],
        3,
    );
    blocked_destination.permissions.can_send_messages = false;
    engine
        .execute(Command::UpsertConversation {
            conversation: blocked_destination,
        })
        .unwrap();
    let blocked_error = engine
        .execute(Command::ForwardMessage {
            source_conversation_id: ConversationId::new("chat:source"),
            message_id: MessageId::new("source:1"),
            destination_conversation_id: ConversationId::new("chat:blocked-forward"),
            local_message_id: MessageId::new("forward:blocked"),
            client_message_id: ClientMessageId("client:forward-blocked".into()),
            sender_id: ActorId::new("human:forwarder"),
            thread_root_message_id: None,
            created_at_ms: 3,
            scheduled_at_ms: None,
            silent: false,
            privacy: ForwardPrivacy::default(),
        })
        .unwrap_err();
    assert_eq!(
        blocked_error,
        EngineError::MessageSendPermissionDenied(ConversationId::new("chat:blocked-forward"))
    );

    let mut channel_destination = Conversation::direct(
        "channel:read-only-forward",
        "Read-only channel",
        vec![participant("human:forwarder", ParticipantRole::Member)],
        3,
    );
    channel_destination.kind = ConversationKind::Channel;
    engine
        .execute(Command::UpsertConversation {
            conversation: channel_destination,
        })
        .unwrap();
    let channel_error = engine
        .execute(Command::ForwardMessage {
            source_conversation_id: ConversationId::new("chat:source"),
            message_id: MessageId::new("source:1"),
            destination_conversation_id: ConversationId::new("channel:read-only-forward"),
            local_message_id: MessageId::new("forward:channel-blocked"),
            client_message_id: ClientMessageId("client:forward-channel-blocked".into()),
            sender_id: ActorId::new("human:forwarder"),
            thread_root_message_id: None,
            created_at_ms: 3,
            scheduled_at_ms: None,
            silent: false,
            privacy: ForwardPrivacy::default(),
        })
        .unwrap_err();
    assert_eq!(
        channel_error,
        EngineError::CommunitySendRestricted(ConversationId::new("channel:read-only-forward"))
    );

    engine
        .execute(Command::QueueMessage {
            conversation_id: ConversationId::new("chat:source"),
            local_message_id: MessageId::new("source:protected"),
            client_message_id: ClientMessageId("client:protected".into()),
            sender_id: ActorId::new("human:forwarder"),
            content: MessageContent::Text {
                text: FormattedText::plain("禁止转发"),
            },
            reply_to_message_id: None,
            thread_root_message_id: None,
            created_at_ms: 4,
            scheduled_at_ms: None,
            silent: false,
            protected_content: true,
        })
        .unwrap();
    let error = engine
        .execute(Command::ForwardMessage {
            source_conversation_id: ConversationId::new("chat:source"),
            message_id: MessageId::new("source:protected"),
            destination_conversation_id: ConversationId::new("chat:destination"),
            local_message_id: MessageId::new("forward:protected"),
            client_message_id: ClientMessageId("client:forward-protected".into()),
            sender_id: ActorId::new("human:forwarder"),
            thread_root_message_id: None,
            created_at_ms: 5,
            scheduled_at_ms: None,
            silent: false,
            privacy: ForwardPrivacy::default(),
        })
        .unwrap_err();
    assert_eq!(error, EngineError::ProtectedContent);
}

#[test]
fn secret_conversations_reject_plaintext_and_protect_encrypted_messages() {
    let mut engine = MessagingEngine::new();
    for (id, name) in [("human:alice", "Alice"), ("human:bob", "Bob")] {
        engine
            .execute(Command::UpsertActor {
                actor: Actor::human(id, name),
            })
            .unwrap();
    }
    let mut conversation = Conversation::direct(
        "secret:contract",
        "Secret",
        vec![
            participant("human:alice", ParticipantRole::Owner),
            participant("human:bob", ParticipantRole::Member),
        ],
        1,
    );
    conversation.kind = ConversationKind::Secret;
    engine
        .execute(Command::UpsertConversation { conversation })
        .unwrap();

    let plaintext_error = engine
        .execute(Command::QueueMessage {
            conversation_id: ConversationId::new("secret:contract"),
            local_message_id: MessageId::new("secret:plain"),
            client_message_id: ClientMessageId("client:plain".into()),
            sender_id: ActorId::new("human:alice"),
            content: MessageContent::Text {
                text: FormattedText::plain("must never persist"),
            },
            reply_to_message_id: None,
            thread_root_message_id: None,
            created_at_ms: 2,
            scheduled_at_ms: None,
            silent: false,
            protected_content: false,
        })
        .unwrap_err();
    assert!(matches!(
        plaintext_error,
        EngineError::SecretPlaintextRejected
    ));

    let alice_private = SecretPrivateKey::generate().unwrap();
    let bob_private = SecretPrivateKey::generate().unwrap();
    let mut alice_session = SecretChatSession::establish(
        ConversationId::new("secret:contract"),
        ActorId::new("human:alice"),
        ActorId::new("human:bob"),
        &alice_private,
        &bob_private.public_key(),
        1,
    )
    .unwrap();
    let envelope = alice_session.encrypt(b"encrypted payload").unwrap();
    engine
        .execute(Command::QueueMessage {
            conversation_id: ConversationId::new("secret:contract"),
            local_message_id: MessageId::new("secret:cipher"),
            client_message_id: ClientMessageId("client:cipher".into()),
            sender_id: ActorId::new("human:alice"),
            content: MessageContent::Secret { envelope },
            reply_to_message_id: None,
            thread_root_message_id: None,
            created_at_ms: 3,
            scheduled_at_ms: None,
            silent: false,
            protected_content: false,
        })
        .unwrap();
    let stored = &engine.state().messages[&ConversationId::new("secret:contract")]
        [&MessageId::new("secret:cipher")];
    assert!(stored.protected_content);
    assert!(matches!(stored.content, MessageContent::Secret { .. }));

    let forward_error = engine
        .execute(Command::ForwardMessage {
            source_conversation_id: ConversationId::new("secret:contract"),
            message_id: MessageId::new("secret:cipher"),
            destination_conversation_id: ConversationId::new("secret:contract"),
            local_message_id: MessageId::new("secret:forward"),
            client_message_id: ClientMessageId("client:forward-secret".into()),
            sender_id: ActorId::new("human:alice"),
            thread_root_message_id: None,
            created_at_ms: 4,
            scheduled_at_ms: None,
            silent: false,
            privacy: ForwardPrivacy::default(),
        })
        .unwrap_err();
    assert!(matches!(forward_error, EngineError::ProtectedContent));
}

#[test]
fn stories_communities_and_bot_execution_enforce_actor_permissions() {
    let mut engine = MessagingEngine::new();
    for actor in [
        Actor::human("human:owner", "Owner"),
        Actor::human("human:member", "Member"),
        Actor::human("human:outsider", "Outsider"),
        Actor::bot("bot:helper", "Helper"),
    ] {
        engine.execute(Command::UpsertActor { actor }).unwrap();
    }

    let mut group = Conversation::direct(
        "group:secure",
        "Secure group",
        vec![
            participant("human:owner", ParticipantRole::Owner),
            participant("human:member", ParticipantRole::Member),
            participant("bot:helper", ParticipantRole::Member),
        ],
        1,
    );
    group.kind = ConversationKind::Group;
    group.owner_id = Some(ActorId::new("human:owner"));
    engine
        .execute(Command::UpsertConversation {
            conversation: group,
        })
        .unwrap();

    let mut community = CommunityState::new(ConversationId::new("group:secure"));
    community.upsert_member(CommunityMember {
        actor_id: ActorId::new("human:owner"),
        status: MemberStatus::Owner,
        admin_title: Some("Owner".into()),
        admin_rights: AdminRights::default(),
        restrictions: MemberRestrictions::default(),
        joined_at_ms: 1,
        invited_by: None,
    });
    engine
        .execute(Command::UpdateCommunity {
            actor_id: ActorId::new("human:owner"),
            community,
        })
        .unwrap();
    let denied = engine
        .execute(Command::SetCommunityMember {
            actor_id: ActorId::new("human:member"),
            conversation_id: ConversationId::new("group:secure"),
            member: CommunityMember {
                actor_id: ActorId::new("human:outsider"),
                status: MemberStatus::Member,
                admin_title: None,
                admin_rights: AdminRights::default(),
                restrictions: MemberRestrictions::default(),
                joined_at_ms: 2,
                invited_by: Some(ActorId::new("human:member")),
            },
        })
        .unwrap_err();
    assert!(matches!(denied, EngineError::CommunityPermissionDenied));

    let story = Story {
        id: StoryId("story:1".into()),
        owner_id: ActorId::new("human:owner"),
        media: MediaRef {
            id: "blob-story-1".into(),
            file_name: Some("story.jpg".into()),
            mime_type: Some("image/jpeg".into()),
            size_bytes: Some(10),
            width: Some(1080),
            height: Some(1920),
            duration_ms: None,
            thumbnail_id: None,
            local_path: None,
            remote_url: Some("fabushi-blob://blob-story-1".into()),
            content_hash: None,
        },
        caption: FormattedText::plain("selected story"),
        privacy: StoryPrivacy {
            kind: StoryPrivacyKind::Selected,
            included_actor_ids: std::collections::BTreeSet::from([ActorId::new("human:member")]),
            excluded_actor_ids: std::collections::BTreeSet::new(),
        },
        created_at_ms: 10,
        expires_at_ms: 100,
        edited_at_ms: None,
        pinned_to_profile: false,
        protected_content: true,
        allow_replies: true,
        views: std::collections::BTreeMap::new(),
    };
    engine
        .execute(Command::PublishStory {
            actor_id: ActorId::new("human:owner"),
            story,
        })
        .unwrap();
    engine
        .execute(Command::ViewStory {
            actor_id: ActorId::new("human:member"),
            story_id: StoryId("story:1".into()),
            viewed_at_ms: 20,
        })
        .unwrap();
    let denied = engine
        .execute(Command::ViewStory {
            actor_id: ActorId::new("human:outsider"),
            story_id: StoryId("story:1".into()),
            viewed_at_ms: 21,
        })
        .unwrap_err();
    assert!(matches!(denied, EngineError::StoryPermissionDenied(_)));

    engine
        .execute(Command::RegisterBot {
            actor_id: ActorId::new("bot:helper"),
            profile: BotProfile {
                actor_id: ActorId::new("bot:helper"),
                description: "Unified helper".into(),
                about: "Runs in the same MessagingService".into(),
                commands: vec![BotCommand {
                    command: "help".into(),
                    description: "Help".into(),
                    scopes: std::collections::BTreeSet::from(["group".into()]),
                }],
                inline_mode_enabled: false,
                inline_placeholder: None,
                groups_allowed: true,
                privacy_mode: false,
                mini_app_id: None,
                payment_provider_ids: Vec::new(),
                business_mode: false,
            },
        })
        .unwrap();
    let invocation = BotInvocation {
        id: "invoke:1".into(),
        bot_id: ActorId::new("bot:helper"),
        sender_id: ActorId::new("human:member"),
        conversation_id: ConversationId::new("group:secure"),
        command: Some("help".into()),
        text: FormattedText::plain("help me"),
        reply_to_message_id: None,
        metadata: std::collections::BTreeMap::new(),
        created_at_ms: 30,
    };
    let events = engine
        .execute(Command::BeginBotInvocation {
            actor_id: ActorId::new("human:member"),
            invocation,
            created_at_ms: 31,
        })
        .unwrap();
    let execution_id = match &events[0] {
        Event::BotRegistryChanged {
            execution: Some(execution),
            ..
        } => execution.id.clone(),
        other => panic!("unexpected bot event: {other:?}"),
    };
    engine
        .execute(Command::FinishBotExecution {
            actor_id: ActorId::new("bot:helper"),
            execution_id,
            success: true,
            finished_at_ms: 32,
            error: None,
        })
        .unwrap();
}

#[test]
fn story_sibling_resolution_fences_stale_suggestions_contacts_visibility_and_media_fallback() {
    let mut engine = MessagingEngine::default();
    let owner_id = ActorId::new("human:story-owner");
    let contact_id = ActorId::new("human:story-contact");
    let close_friend_id = ActorId::new("human:story-close-friend");
    let stranger_id = ActorId::new("human:story-stranger");
    let foreign_owner_id = ActorId::new("human:foreign-owner");

    let mut owner = Actor::human("human:story-owner", "Story Owner");
    owner.capabilities.push("contact:human:story-contact".into());
    owner
        .capabilities
        .push("close_friend:human:story-close-friend".into());

    for actor in [
        owner,
        Actor::human("human:story-contact", "Contact"),
        Actor::human("human:story-close-friend", "Close Friend"),
        Actor::human("human:story-stranger", "Stranger"),
        Actor::human("human:foreign-owner", "Foreign Owner"),
    ] {
        engine.execute(Command::UpsertActor { actor }).unwrap();
    }

    let make_story = |id: &str, owner: &ActorId, kind: StoryPrivacyKind, created: i64, expires: i64| Story {
        id: StoryId(id.into()),
        owner_id: owner.clone(),
        media: MediaRef {
            id: format!("media:{id}"),
            file_name: Some(format!("{id}.mp4")),
            mime_type: Some("video/mp4".into()),
            size_bytes: Some(1024),
            width: Some(1080),
            height: Some(1920),
            duration_ms: Some(5000),
            thumbnail_id: Some(format!("thumb:{id}")),
            local_path: None,
            remote_url: Some(format!("fabushi-blob://media:{id}")),
            content_hash: None,
        },
        caption: FormattedText::plain(id),
        privacy: StoryPrivacy {
            kind,
            included_actor_ids: Default::default(),
            excluded_actor_ids: Default::default(),
        },
        created_at_ms: created,
        expires_at_ms: expires,
        edited_at_ms: None,
        pinned_to_profile: false,
        protected_content: false,
        allow_replies: true,
        views: Default::default(),
    };

    for (actor_id, story) in [
        (
            owner_id.clone(),
            make_story("story:contacts:1", &owner_id, StoryPrivacyKind::Contacts, 10, 100),
        ),
        (
            owner_id.clone(),
            make_story("story:contacts:2", &owner_id, StoryPrivacyKind::Contacts, 20, 100),
        ),
        (
            owner_id.clone(),
            make_story("story:close:3", &owner_id, StoryPrivacyKind::CloseFriends, 30, 100),
        ),
        (
            foreign_owner_id.clone(),
            make_story("story:foreign:1", &foreign_owner_id, StoryPrivacyKind::Everyone, 15, 100),
        ),
    ] {
        engine
            .execute(Command::PublishStory { actor_id, story })
            .unwrap();
    }

    // Contact can view Contacts story, but Stranger cannot; Contact cannot view CloseFriends story, CloseFriend can
    engine
        .execute(Command::ViewStory {
            actor_id: contact_id.clone(),
            story_id: StoryId("story:contacts:1".into()),
            viewed_at_ms: 25,
        })
        .unwrap();
    assert!(matches!(
        engine
            .execute(Command::ViewStory {
                actor_id: stranger_id.clone(),
                story_id: StoryId("story:contacts:1".into()),
                viewed_at_ms: 25,
            })
            .unwrap_err(),
        EngineError::StoryPermissionDenied(_)
    ));
    assert!(matches!(
        engine
            .execute(Command::ViewStory {
                actor_id: contact_id.clone(),
                story_id: StoryId("story:close:3".into()),
                viewed_at_ms: 25,
            })
            .unwrap_err(),
        EngineError::StoryPermissionDenied(_)
    ));
    engine
        .execute(Command::ViewStory {
            actor_id: close_friend_id.clone(),
            story_id: StoryId("story:close:3".into()),
            viewed_at_ms: 26,
        })
        .unwrap();
    engine
        .execute(Command::ReactStory {
            actor_id: close_friend_id.clone(),
            story_id: StoryId("story:close:3".into()),
            reaction: Some("❤️".into()),
        })
        .unwrap();

    // LookupShownId (resolve_shown_story_id) fences foreign/expired/unauthorized suggestions and falls back to first unread story
    let (contact_flag, close_flag) = engine.story_visibility_flags(&owner_id, &contact_id);
    assert_eq!((contact_flag, close_flag), (true, false));

    // Suggested foreign story id is fenced and falls back to first unread story for contact ("story:contacts:2", since "story:contacts:1" was viewed)
    let resolved_after_foreign_suggestion = resolve_shown_story_id(
        engine.state().stories.values(),
        &owner_id,
        &contact_id,
        contact_flag,
        close_flag,
        50,
        Some(&StoryId("story:foreign:1".into())),
    );
    assert_eq!(
        resolved_after_foreign_suggestion,
        Some(StoryId("story:contacts:2".into()))
    );

    // Suggested valid same-owner visible story id is preserved even if already viewed
    let resolved_explicit_valid = resolve_shown_story_id(
        engine.state().stories.values(),
        &owner_id,
        &contact_id,
        contact_flag,
        close_flag,
        50,
        Some(&StoryId("story:contacts:1".into())),
    );
    assert_eq!(
        resolved_explicit_valid,
        Some(StoryId("story:contacts:1".into()))
    );

    // Media/thumbnail fallback state machine and generation token fencing
    let story_media = &engine.state().stories[&StoryId("story:contacts:1".into())].media;
    let mut media_state = StorySiblingMediaState::from_media(story_media);
    assert_eq!(
        media_state.active_display_resource_id(),
        Some("thumb:story:contacts:1")
    );
    assert!(media_state.begin_load());
    assert!(!media_state.begin_load());
    assert!(!media_state.complete_load("media:wrong"));
    media_state.fail_stream_or_load();
    assert_eq!(
        media_state,
        StorySiblingMediaState::TerminalFallback {
            thumbnail_id: Some("thumb:story:contacts:1".into()),
        }
    );
    assert!(!media_state.begin_load());

    let token = StoryResolutionToken {
        owner_id: owner_id.clone(),
        story_id: StoryId("story:contacts:1".into()),
        generation: 7,
    };
    assert!(token.matches_callback(&owner_id, &StoryId("story:contacts:1".into()), 7));
    assert!(!token.matches_callback(&owner_id, &StoryId("story:contacts:1".into()), 6));
    assert!(!token.matches_callback(&foreign_owner_id, &StoryId("story:contacts:1".into()), 7));
}

#[test]
fn send_menu_mode_and_applicability_enforces_paid_silent_only_reminder_and_scheduled_to_user() {
    let mut engine = MessagingEngine::new();
    let sender_id = ActorId::new("human:sender");
    let peer_offline_id = ActorId::new("human:peer-offline");
    let peer_paid_id = ActorId::new("human:peer-paid");
    let mut peer_paid = Actor::human("human:peer-paid", "Paid Peer");
    peer_paid.capabilities = vec!["paid_messages_required".into()];

    for actor in [
        Actor::human("human:sender", "Sender"),
        Actor::human("human:peer-offline", "Offline Peer"),
        peer_paid,
    ] {
        engine.execute(Command::UpsertActor { actor }).unwrap();
    }

    let mut saved_messages = Conversation::direct(
        "chat:saved",
        "Saved Messages",
        vec![participant("human:sender", ParticipantRole::Owner)],
        1,
    );
    saved_messages.kind = ConversationKind::SavedMessages;

    let direct_offline = Conversation::direct(
        "chat:direct-offline",
        "Offline Peer",
        vec![
            participant("human:sender", ParticipantRole::Owner),
            participant("human:peer-offline", ParticipantRole::Member),
        ],
        1,
    );
    let direct_paid = Conversation::direct(
        "chat:direct-paid",
        "Paid Peer",
        vec![
            participant("human:sender", ParticipantRole::Owner),
            participant("human:peer-paid", ParticipantRole::Member),
        ],
        1,
    );

    for conversation in [saved_messages, direct_offline, direct_paid] {
        engine
            .execute(Command::UpsertConversation { conversation })
            .unwrap();
    }

    // 1. Empty destination selection -> Disabled
    let disabled = engine.resolve_send_menu_details(&sender_id, &[]);
    assert_eq!(disabled.mode, SendMenuMode::Disabled);
    assert!(!disabled.accepts_options(true, None, false));
    assert!(!disabled.accepts_options(false, Some(100), false));
    assert!(!disabled.accepts_options(false, None, true));

    // 2. Single SavedMessages destination -> Reminder (suppresses silent and when_online, allows schedule/reminder)
    let reminder =
        engine.resolve_send_menu_details(&sender_id, &[ConversationId::new("chat:saved")]);
    assert_eq!(reminder.mode, SendMenuMode::Reminder);
    assert!(!reminder.allow_silent);
    assert!(reminder.allow_schedule);
    assert!(!reminder.allow_when_online);
    assert!(reminder.allow_reminder);
    assert!(!reminder.accepts_options(true, Some(100), false));
    assert_eq!(
        reminder.normalize_options(true, Some(100), false),
        Some((false, Some(100), false))
    );

    // QueueMessage and ForwardMessage to SavedMessages clamp silent=false
    engine
        .execute(Command::QueueMessage {
            conversation_id: ConversationId::new("chat:saved"),
            local_message_id: MessageId::new("saved:1"),
            client_message_id: ClientMessageId("client:saved:1".into()),
            sender_id: sender_id.clone(),
            content: MessageContent::Text {
                text: FormattedText::plain("Self reminder"),
            },
            reply_to_message_id: None,
            thread_root_message_id: None,
            created_at_ms: 10,
            scheduled_at_ms: Some(100),
            silent: true,
            protected_content: false,
        })
        .unwrap();
    assert!(!engine.state().messages[&ConversationId::new("chat:saved")][&MessageId::new("saved:1")].silent);

    // 3. Single offline human destination with visible LastSeen -> ScheduledToUser
    let scheduled_to_user = engine
        .resolve_send_menu_details(&sender_id, &[ConversationId::new("chat:direct-offline")]);
    assert_eq!(scheduled_to_user.mode, SendMenuMode::ScheduledToUser);
    assert!(scheduled_to_user.allow_silent);
    assert!(scheduled_to_user.allow_schedule);
    assert!(scheduled_to_user.allow_when_online);
    assert!(!scheduled_to_user.allow_reminder);
    assert!(scheduled_to_user.accepts_options(true, None, true));
    assert!(!scheduled_to_user.accepts_options(false, Some(100), true));

    // 4. Hiding LastSeen privacy downgrades ScheduledToUser -> Scheduled (suppressing when_online)
    let mut hidden_last_seen = PrivacySettings::default();
    hidden_last_seen.rules.insert(
        PrivacyKey::LastSeen,
        PrivacyRule {
            visibility: PrivacyVisibility::Nobody,
            allow_actors: Vec::new(),
            disallow_actors: Vec::new(),
        },
    );
    engine
        .execute(Command::UpsertPrivacySettings {
            actor_id: peer_offline_id.clone(),
            settings: hidden_last_seen,
        })
        .unwrap();
    let downgraded = engine
        .resolve_send_menu_details(&sender_id, &[ConversationId::new("chat:direct-offline")]);
    assert_eq!(downgraded.mode, SendMenuMode::Scheduled);
    assert!(!downgraded.allow_when_online);

    // 5. Paid destination constrains send menu to SilentOnly (even when combined with non-paid destination)
    let paid_menu = engine.resolve_send_menu_details(
        &sender_id,
        &[
            ConversationId::new("chat:direct-offline"),
            ConversationId::new("chat:direct-paid"),
        ],
    );
    assert_eq!(paid_menu.mode, SendMenuMode::SilentOnly);
    assert!(paid_menu.allow_silent);
    assert!(!paid_menu.allow_schedule);
    assert!(!paid_menu.allow_when_online);
    assert!(paid_menu.accepts_options(true, None, false));
    assert!(!paid_menu.accepts_options(false, Some(200), false));

    // 6. SavedSublist child for paid participant is SilentOnly unless no_paid_messages exception is set
    let saved_sublist_dest = ConversationDestination {
        conversation_id: ConversationId::new("chat:saved"),
        child: Some(ConversationChildIdentity::SavedSublist {
            participant_id: peer_paid_id,
        }),
    };
    let sublist_before_exception = engine
        .resolve_destination_send_menu_details(&sender_id, std::slice::from_ref(&saved_sublist_dest));
    assert_eq!(sublist_before_exception.mode, SendMenuMode::SilentOnly);

    engine
        .execute(Command::SetConversationChildNoPaidMessages {
            destination: saved_sublist_dest.clone(),
            actor_id: sender_id.clone(),
            no_paid_messages: true,
        })
        .unwrap();
    let sublist_after_exception =
        engine.resolve_destination_send_menu_details(&sender_id, &[saved_sublist_dest]);
    assert_eq!(sublist_after_exception.mode, SendMenuMode::Scheduled);
}
