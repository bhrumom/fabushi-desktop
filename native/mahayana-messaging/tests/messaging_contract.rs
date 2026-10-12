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
fn checkout_validates_requested_customer_information_before_charging() {
    let mut engine = MessagingEngine::new();
    engine
        .execute(Command::UpsertActor {
            actor: Actor::human("buyer:validated", "购买者"),
        })
        .unwrap();
    engine
        .execute(Command::UpsertActor {
            actor: Actor::bot("seller:validated", "商店机器人"),
        })
        .unwrap();
    engine
        .execute(Command::UpsertConversation {
            conversation: Conversation::direct(
                "chat:validated-checkout",
                "验证结算",
                vec![
                    participant("buyer:validated", ParticipantRole::Owner),
                    participant("seller:validated", ParticipantRole::Member),
                ],
                1,
            ),
        })
        .unwrap();

    let invoice = Invoice {
        id: "invoice:validated".into(),
        conversation_id: ConversationId::new("chat:validated-checkout"),
        seller_id: ActorId::new("seller:validated"),
        title: "Validated checkout".into(),
        description: "requested customer fields".into(),
        kind: InvoiceKind::OneTime,
        currency: "USD".into(),
        prices: vec![PriceLine {
            label: "Order".into(),
            amount: Money::new("USD", 999),
        }],
        payload: "validated".into(),
        provider_id: "fabushi-pay".into(),
        start_parameter: None,
        request_name: true,
        request_email: true,
        request_phone: true,
        request_shipping_address: true,
        flexible_shipping: false,
        created_at_ms: 2,
        expires_at_ms: None,
    };
    engine
        .execute(Command::CreateInvoice {
            invoice: invoice.clone(),
        })
        .unwrap();

    let missing = engine
        .execute(Command::CheckoutInvoice {
            invoice_id: invoice.id.clone(),
            order_id: "order:missing-customer".into(),
            buyer_id: ActorId::new("buyer:validated"),
            customer: None,
            created_at_ms: 3,
        })
        .unwrap_err();
    assert_eq!(missing, EngineError::InvalidCustomerInfo);

    let invalid_customer = CustomerInfo {
        name: Some("Buyer".into()),
        email: Some("buyer@example.com".into()),
        phone: Some("+16025550123".into()),
        shipping_address: Some(ShippingAddress {
            country_code: "US".into(),
            state: "AZ".into(),
            city: "P".into(),
            street_line1: "1 Main Street".into(),
            street_line2: None,
            postal_code: "85001".into(),
        }),
    };
    assert!(!invalid_customer.is_valid_for_invoice(&invoice));
    let invalid = engine
        .execute(Command::CheckoutInvoice {
            invoice_id: invoice.id.clone(),
            order_id: "order:invalid-customer".into(),
            buyer_id: ActorId::new("buyer:validated"),
            customer: Some(invalid_customer),
            created_at_ms: 3,
        })
        .unwrap_err();
    assert_eq!(invalid, EngineError::InvalidCustomerInfo);

    engine
        .execute(Command::CreditWalletSettlement {
            request_id: "settlement:validated".into(),
            owner_id: ActorId::new("buyer:validated"),
            amount: Money::new("USD", 999),
            reference: Some("checkout-fixture".into()),
            settled_at_ms: 3,
        })
        .unwrap();

    let valid_customer = CustomerInfo {
        name: Some("Buyer".into()),
        email: Some("buyer@example.com".into()),
        phone: Some("+16025550123".into()),
        shipping_address: Some(ShippingAddress {
            country_code: "US".into(),
            state: "AZ".into(),
            city: "Phoenix".into(),
            street_line1: "1 Main Street".into(),
            street_line2: Some(String::new()),
            postal_code: "85001".into(),
        }),
    };
    assert!(valid_customer.is_valid_for_invoice(&invoice));
    let events = engine
        .execute(Command::CheckoutInvoice {
            invoice_id: invoice.id.clone(),
            order_id: "order:validated".into(),
            buyer_id: ActorId::new("buyer:validated"),
            customer: Some(valid_customer.clone()),
            created_at_ms: 4,
        })
        .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::OrderUpserted { order }
            if order.id == "order:validated"
                && order.customer.as_ref() == Some(&valid_customer)
    )));
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
            destination: ConversationDestination::root(ConversationId::new("chat:destination")),
            local_message_id: MessageId::new("forward:1"),
            client_message_id: ClientMessageId("client:forward".into()),
            sender_id: ActorId::new("human:forwarder"),
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
    assert_eq!(forwarded.thread_root_message_id, None);
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
            destination: ConversationDestination::root(ConversationId::new("chat:destination")),
            local_message_id: MessageId::new("forward:privacy"),
            client_message_id: ClientMessageId("client:forward-privacy".into()),
            sender_id: ActorId::new("human:forwarder"),
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
            destination: ConversationDestination::root(ConversationId::new("chat:blocked-forward")),
            local_message_id: MessageId::new("forward:blocked"),
            client_message_id: ClientMessageId("client:forward-blocked".into()),
            sender_id: ActorId::new("human:forwarder"),
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
            destination: ConversationDestination::root(ConversationId::new("channel:read-only-forward")),
            local_message_id: MessageId::new("forward:channel-blocked"),
            client_message_id: ClientMessageId("client:forward-channel-blocked".into()),
            sender_id: ActorId::new("human:forwarder"),
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
            destination: ConversationDestination::root(ConversationId::new("chat:destination")),
            local_message_id: MessageId::new("forward:protected"),
            client_message_id: ClientMessageId("client:forward-protected".into()),
            sender_id: ActorId::new("human:forwarder"),
            created_at_ms: 5,
            scheduled_at_ms: None,
            silent: false,
            privacy: ForwardPrivacy::default(),
        })
        .unwrap_err();
    assert_eq!(error, EngineError::ProtectedContent);
}

#[test]
fn forwarding_uses_canonical_typed_child_destinations_and_fails_closed_after_destroy() {
    let owner = ActorId::new("human:typed-forward-owner");
    let peer = ActorId::new("human:typed-forward-peer");
    let mut engine = MessagingEngine::new();
    for actor in [
        Actor::human(owner.0.clone(), "Owner"),
        Actor::human(peer.0.clone(), "Peer"),
    ] {
        engine.execute(Command::UpsertActor { actor }).unwrap();
    }

    let owner_participant = Participant {
        actor_id: owner.clone(),
        role: ParticipantRole::Owner,
        joined_at_ms: 1,
        muted_until_ms: None,
    };
    for (id, title) in [
        ("chat:typed-forward-source", "Source"),
        ("saved:typed-forward", "Saved"),
        ("parent:typed-forward", "Parent"),
        ("child:typed-forward", "Child"),
    ] {
        let mut conversation =
            Conversation::direct(id, title, vec![owner_participant.clone()], 1);
        if id == "saved:typed-forward" {
            conversation.kind = ConversationKind::SavedMessages;
            conversation.owner_id = Some(owner.clone());
        }
        engine.execute(Command::UpsertConversation { conversation }).unwrap();
    }

    engine
        .execute(Command::QueueMessage {
            conversation_id: ConversationId::new("chat:typed-forward-source"),
            local_message_id: MessageId::new("typed-forward:source"),
            client_message_id: ClientMessageId("client:typed-forward-source".into()),
            sender_id: owner.clone(),
            content: MessageContent::Text {
                text: FormattedText::plain("typed destination"),
            },
            reply_to_message_id: None,
            thread_root_message_id: None,
            created_at_ms: 2,
            scheduled_at_ms: None,
            silent: false,
            protected_content: false,
        })
        .unwrap();

    let saved = ConversationDestination::saved_sublist(
        ConversationId::new("saved:typed-forward"),
        peer,
    );
    engine
        .execute(Command::SetConversationChildActive {
            destination: saved.clone(),
            actor_id: owner.clone(),
            active: true,
        })
        .unwrap();
    assert_eq!(
        engine
            .execute(Command::ForwardMessage {
                source_conversation_id: ConversationId::new("chat:typed-forward-source"),
                message_id: MessageId::new("typed-forward:source"),
                destination: saved.clone(),
                local_message_id: MessageId::new("typed-forward:saved-before-authority"),
                client_message_id: ClientMessageId(
                    "client:typed-forward-saved-before-authority".into(),
                ),
                sender_id: owner.clone(),
                created_at_ms: 3,
                scheduled_at_ms: None,
                silent: false,
                privacy: ForwardPrivacy::default(),
            })
            .unwrap_err(),
        EngineError::ConversationChildMessageMismatch
    );

    engine
        .execute(Command::ReconcileSavedSublistMembership {
            destination: saved.clone(),
            actor_id: owner.clone(),
            message_ids: Vec::new(),
        })
        .unwrap();
    engine
        .execute(Command::ForwardMessage {
            source_conversation_id: ConversationId::new("chat:typed-forward-source"),
            message_id: MessageId::new("typed-forward:source"),
            destination: saved.clone(),
            local_message_id: MessageId::new("typed-forward:saved"),
            client_message_id: ClientMessageId("client:typed-forward-saved".into()),
            sender_id: owner.clone(),
            created_at_ms: 4,
            scheduled_at_ms: None,
            silent: true,
            privacy: ForwardPrivacy::default(),
        })
        .unwrap();
    let saved_state = engine
        .state()
        .conversation_child_states
        .iter()
        .find(|child| child.destination == saved && child.actor_id == owner)
        .expect("saved child state");
    assert!(saved_state.authoritative_membership_known);
    assert!(saved_state.has_authoritative_message("typed-forward:saved"));

    engine
        .execute(Command::DestroyConversationChild {
            destination: saved.clone(),
            actor_id: owner.clone(),
        })
        .unwrap();
    assert_eq!(
        engine
            .execute(Command::ForwardMessage {
                source_conversation_id: ConversationId::new("chat:typed-forward-source"),
                message_id: MessageId::new("typed-forward:source"),
                destination: saved,
                local_message_id: MessageId::new("typed-forward:saved-stale"),
                client_message_id: ClientMessageId("client:typed-forward-saved-stale".into()),
                sender_id: owner.clone(),
                created_at_ms: 5,
                scheduled_at_ms: None,
                silent: false,
                privacy: ForwardPrivacy::default(),
            })
            .unwrap_err(),
        EngineError::InvalidConversationChildDestination
    );

    let nested = ConversationDestination::nested_conversation(
        ConversationId::new("parent:typed-forward"),
        ConversationId::new("child:typed-forward"),
    );
    engine
        .execute(Command::SetConversationChildActive {
            destination: nested.clone(),
            actor_id: owner.clone(),
            active: true,
        })
        .unwrap();
    engine
        .execute(Command::ForwardMessage {
            source_conversation_id: ConversationId::new("chat:typed-forward-source"),
            message_id: MessageId::new("typed-forward:source"),
            destination: nested,
            local_message_id: MessageId::new("typed-forward:nested"),
            client_message_id: ClientMessageId("client:typed-forward-nested".into()),
            sender_id: owner,
            created_at_ms: 6,
            scheduled_at_ms: None,
            silent: false,
            privacy: ForwardPrivacy::default(),
        })
        .unwrap();
    assert!(
        engine.state().messages[&ConversationId::new("child:typed-forward")]
            .contains_key(&MessageId::new("typed-forward:nested"))
    );
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
            destination: ConversationDestination::root(ConversationId::new("secret:contract")),
            local_message_id: MessageId::new("secret:forward"),
            client_message_id: ClientMessageId("client:forward-secret".into()),
            sender_id: ActorId::new("human:alice"),
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
        anonymous_view_count: 0,
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
fn recent_participant_selection_projection_is_durable_bounded_account_scoped_and_cursor_neutral() {
    let owner_a = ActorId::new("human:recent-integration-owner-a");
    let owner_b = ActorId::new("human:recent-integration-owner-b");
    let mut service = MessagingService::load(MemoryStateStore::default()).unwrap();

    for index in 0_i64..=48 {
        let peer = ActorId::new(format!("human:recent-integration-peer:{index}"));
        service
            .note_recent_participant_selection(
                Actor::human(owner_a.0.clone(), "Owner A"),
                Actor::human(peer.0.clone(), format!("Peer {index}")),
                100 + index,
            )
            .unwrap();
    }

    let repeated = ActorId::new("human:recent-integration-peer:5");
    service
        .note_recent_participant_selection(
            Actor::human(owner_a.0.clone(), "Owner A"),
            Actor::human(repeated.0.clone(), "Peer 5"),
            200,
        )
        .unwrap();
    service
        .note_recent_participant_selection(
            Actor::human(owner_b.0.clone(), "Owner B"),
            Actor::human(repeated.0.clone(), "Peer 5"),
            201,
        )
        .unwrap();

    let recent_a = service.engine().recent_participant_suggestions(&owner_a);
    assert_eq!(recent_a.len(), 48);
    assert_eq!(recent_a.first(), Some(&repeated));
    assert_eq!(
        recent_a.iter().filter(|candidate| *candidate == &repeated).count(),
        1
    );
    assert_eq!(
        service.engine().recent_participant_suggestions(&owner_b),
        &[repeated.clone()]
    );

    let before_invalid = service.engine().state().clone();
    assert!(service
        .note_recent_participant_suggestion(owner_a.clone(), owner_a.clone(), 202)
        .is_err());
    assert!(service
        .note_recent_participant_suggestion(
            owner_a.clone(),
            ActorId::new("human:recent-integration-unknown"),
            203,
        )
        .is_err());
    assert_eq!(service.engine().state(), &before_invalid);

    let store = service.into_store();
    let snapshot = store.load().unwrap().expect("recent projection snapshot");
    assert_eq!(
        snapshot.cursor, 0,
        "projection-only writes must not advance the cursor or journal"
    );

    let reopened = MessagingService::load(store).unwrap();
    assert_eq!(
        reopened
            .engine()
            .recent_participant_suggestions(&owner_a)
            .first(),
        Some(&repeated)
    );
    assert_eq!(
        reopened.engine().recent_participant_suggestions(&owner_b),
        &[repeated]
    );
}


#[test]
fn shared_media_projection_uses_canonical_message_and_child_state_only() {
    let actor_id = ActorId::new("human:shared-media-contract-owner");
    let parent = ConversationId::new("conversation:shared-media-contract");
    let saved = ConversationDestination::saved_sublist(
        parent.clone(),
        ActorId::new("human:shared-media-contract-peer"),
    );
    let media_id = MessageId::new("message:shared-media-contract");
    let text_id = MessageId::new("message:shared-media-contract-text");

    let mut child =
        ConversationChildRuntimeState::new(saved.clone(), actor_id.clone()).unwrap();
    assert!(child.reconcile_authoritative_message_ids(vec![media_id.0.clone()]));

    let media = Message {
        id: media_id.clone(),
        conversation_id: parent.clone(),
        sender_id: ActorId::new("human:peer"),
        content: MessageContent::Photo {
            media: MediaRef {
                id: "media:shared-media-contract".into(),
                file_name: Some("shared.jpg".into()),
                mime_type: Some("image/jpeg".into()),
                size_bytes: Some(256),
                width: Some(32),
                height: Some(32),
                duration_ms: None,
                thumbnail_id: None,
                local_path: None,
                remote_url: None,
                content_hash: None,
            },
            caption: FormattedText::plain(""),
            spoiler: false,
        },
        reply_to_message_id: None,
        thread_root_message_id: None,
        forward_origin: None,
        reply_markup: None,
        reactions: Vec::new(),
        delivery_state: DeliveryState::Delivered,
        created_at_ms: 10,
        edited_at_ms: None,
        scheduled_at_ms: None,
        silent: false,
        protected_content: false,
        pinned: false,
        deleted: false,
    };
    let text = Message {
        id: text_id,
        conversation_id: parent.clone(),
        sender_id: ActorId::new("human:peer"),
        content: MessageContent::Text {
            text: FormattedText::plain("not shared media"),
        },
        reply_to_message_id: None,
        thread_root_message_id: None,
        forward_origin: None,
        reply_markup: None,
        reactions: Vec::new(),
        delivery_state: DeliveryState::Delivered,
        created_at_ms: 11,
        edited_at_ms: None,
        scheduled_at_ms: None,
        silent: false,
        protected_content: false,
        pinned: false,
        deleted: false,
    };

    let mut state = MessagingState::default();
    state.actors.insert(
        actor_id.clone(),
        Actor::human(actor_id.0.clone(), "Shared Media Contract Owner"),
    );
    state.conversation_child_states.push(child);
    state
        .messages
        .entry(parent.clone())
        .or_default()
        .insert(media.id.clone(), media);
    state
        .messages
        .entry(parent.clone())
        .or_default()
        .insert(text.id.clone(), text);

    let mut engine = MessagingEngine::from_state(state);
    assert_eq!(
        engine.shared_media_message_ids(&saved, &actor_id).unwrap(),
        vec![media_id.clone()]
    );
    assert_eq!(
        engine
            .shared_media_message_ids_for_kind(&saved, &actor_id, SharedMediaKind::Photo)
            .unwrap(),
        vec![media_id.clone()],
        "the public contract must expose the upstream-equivalent typed media projection"
    );

    engine.apply(Event::MessagesDeleted {
        conversation_id: parent,
        message_ids: vec![media_id],
    });
    assert!(
        engine
            .shared_media_message_ids(&saved, &actor_id)
            .unwrap()
            .is_empty(),
        "canonical deletion must invalidate the derived shared-media projection without a second cache"
    );
}

#[test]
fn shared_media_search_deletion_rebuilds_from_canonical_state_without_evicting_media_cache() {
    let actor_id = ActorId::new("human:shared-media-search-owner");
    let conversation_id = ConversationId::new("conversation:shared-media-search");
    let message_id = MessageId::new("message:shared-media-search");
    let media = MediaRef {
        id: "media:shared-media-search".into(),
        file_name: Some("searchable.jpg".into()),
        mime_type: Some("image/jpeg".into()),
        size_bytes: Some(512),
        width: Some(64),
        height: Some(64),
        duration_ms: None,
        thumbnail_id: None,
        local_path: None,
        remote_url: None,
        content_hash: None,
    };
    let conversation = Conversation::direct(
        conversation_id.0.clone(),
        "Shared Media Search",
        vec![participant(&actor_id.0, ParticipantRole::Owner)],
        1,
    );
    let message = Message {
        id: message_id.clone(),
        conversation_id: conversation_id.clone(),
        sender_id: actor_id.clone(),
        content: MessageContent::Photo {
            media: media.clone(),
            caption: FormattedText::plain("searchable shared media"),
            spoiler: false,
        },
        reply_to_message_id: None,
        thread_root_message_id: None,
        forward_origin: None,
        reply_markup: None,
        reactions: Vec::new(),
        delivery_state: DeliveryState::Delivered,
        created_at_ms: 10,
        edited_at_ms: None,
        scheduled_at_ms: None,
        silent: false,
        protected_content: false,
        pinned: false,
        deleted: false,
    };

    let mut state = MessagingState::default();
    state.actors.insert(
        actor_id.clone(),
        Actor::human(actor_id.0.clone(), "Shared Media Search Owner"),
    );
    state
        .conversations
        .insert(conversation_id.clone(), conversation);
    state
        .messages
        .entry(conversation_id.clone())
        .or_default()
        .insert(message_id.clone(), message);

    let mut store = MemoryStateStore::default();
    store
        .save(&MessagingSnapshot::new(state, 0, 10))
        .expect("seed canonical messaging state");
    let mut service = MessagingService::load(store).expect("load canonical messaging service");
    let context = |request_id: &str| RequestContext {
        request_id: request_id.into(),
        device_id: "desktop:shared-media-search".into(),
        actor_id: actor_id.clone(),
        session_id: "session:shared-media-search".into(),
        sent_at_ms: 10,
    };
    let query = SearchQuery {
        text: "searchable".into(),
        scope: SearchScope::Media,
        conversation_id: Some(conversation_id.clone()),
        sender_id: None,
        from_ms: None,
        to_ms: None,
        limit: 20,
    };

    let before = service
        .handle(
            ClientEnvelope::new(
                context("request:shared-media-search-before"),
                ClientCommand::Search {
                    query: query.clone(),
                },
            ),
            11,
        )
        .expect("search canonical shared media before deletion");
    let ServerEvent::SearchResults { results, .. } = &before[0].event else {
        panic!("expected shared-media search results");
    };
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, message_id.0);

    let mut media_cache = MediaCache::new(1024).expect("create independent media cache");
    media_cache
        .insert(media, "/cache/shared-media-search.jpg", 512, 11, false)
        .expect("materialize media independently from search projection");

    service
        .handle(
            ClientEnvelope::new(
                context("request:shared-media-search-delete"),
                ClientCommand::DeleteMessages {
                    conversation_id: conversation_id.clone(),
                    message_ids: vec![message_id],
                    for_everyone: false,
                },
            ),
            12,
        )
        .expect("delete canonical message");

    let after = service
        .handle(
            ClientEnvelope::new(
                context("request:shared-media-search-after"),
                ClientCommand::Search { query },
            ),
            13,
        )
        .expect("rebuild shared-media search after deletion");
    let ServerEvent::SearchResults { results, .. } = &after[0].event else {
        panic!("expected shared-media search results after deletion");
    };
    assert!(
        results.is_empty(),
        "shipping search must rebuild from canonical state and exclude deleted media"
    );
    assert!(
        media_cache.entries().contains_key("media:shared-media-search"),
        "derived projection invalidation must not transfer ownership of physical media cache bytes"
    );
}

