use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::session::gateway::{
    SessionGatewayError, dispatch_production_session_gateway_call,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use serde_json::{Value, json};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fabushi-session-group-{label}-{}-{suffix}",
        std::process::id()
    ));
    let _ = std::fs::create_dir_all(&path);
    path
}

fn runtime_for_user(root: &std::path::Path, user_id: &str) -> Arc<ProductionSessionWorkers> {
    let agents = root.join("agents");
    let memory = Arc::new(
        mahayana_host_runtime::extensions::memory::memory_service::MemoryService::new(
            agents.clone(),
        ),
    );
    Arc::new(
        ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
            &agents,
            500,
            Some(user_id.to_string()),
            Arc::new(|| None),
            memory,
        ),
    )
}

fn dispatch_ok(
    runtime: &Arc<ProductionSessionWorkers>,
    method: &str,
    args: Value,
) -> Value {
    dispatch_production_session_gateway_call(runtime, method, &args)
        .unwrap_or_else(|| panic!("unhandled method {method}"))
        .unwrap_or_else(|err| panic!("dispatch {method} failed: {err:?}"))
}

fn dispatch_err(
    runtime: &Arc<ProductionSessionWorkers>,
    method: &str,
    args: Value,
) -> SessionGatewayError {
    dispatch_production_session_gateway_call(runtime, method, &args)
        .unwrap_or_else(|| panic!("unhandled method {method}"))
        .expect_err(&format!("expected dispatch {method} to fail, but it succeeded"))
}

#[test]
fn group_and_channel_creation_listing_and_direct_zero_regression() {
    let root = temp_root("creation-listing");
    let alice = runtime_for_user(&root, "user-alice");

    // 1. Create a 1:1 direct human conversation
    let direct = dispatch_ok(
        &alice,
        "createHumanConversation",
        json!({
            "peerHumanId": "user-bob",
            "title": "Alice and Bob"
        }),
    );
    assert_eq!(direct["kind"], "human");
    let direct_id = direct["id"].as_str().expect("direct id").to_string();
    assert!(direct_id.starts_with("human-direct-"));

    // 2. Create a Group conversation
    let group = dispatch_ok(
        &alice,
        "createGroupConversation",
        json!({
            "groupId": "engineers-core",
            "title": "Core Engineering",
            "participantIds": ["user-bob", "user-charlie"],
            "slowModeSeconds": 10
        }),
    );
    assert_eq!(group["kind"], "group");
    assert_eq!(group["id"], "group-engineers-core");
    assert_eq!(group["title"], "Core Engineering");
    assert_eq!(group["slowModeSeconds"], 10);
    let participants = group["participantIds"].as_array().expect("participants");
    assert!(participants.iter().any(|p| p == "user-alice"));
    assert!(participants.iter().any(|p| p == "user-bob"));
    assert!(participants.iter().any(|p| p == "user-charlie"));

    // 3. Create a Channel conversation
    let channel = dispatch_ok(
        &alice,
        "createChannelConversation",
        json!({
            "channelId": "announcements",
            "title": "Company Announcements",
            "participantIds": ["user-bob"]
        }),
    );
    assert_eq!(channel["kind"], "channel");
    assert_eq!(channel["id"], "channel-announcements");
    assert_eq!(channel["title"], "Company Announcements");

    // 4. Verify listHumanConversations lists all three with distinct kinds
    let list = dispatch_ok(&alice, "listHumanConversations", json!({}));
    let items = list.as_array().expect("conversations array");
    assert_eq!(items.len(), 3);

    let direct_item = items.iter().find(|c| c["id"] == direct_id).expect("direct item");
    assert_eq!(direct_item["kind"], "human");

    let group_item = items.iter().find(|c| c["id"] == "group-engineers-core").expect("group item");
    assert_eq!(group_item["kind"], "group");

    let channel_item = items.iter().find(|c| c["id"] == "channel-announcements").expect("channel item");
    assert_eq!(channel_item["kind"], "channel");
}

#[test]
fn community_send_policy_member_restrictions_and_channel_posting() {
    let root = temp_root("send-policy");
    let alice = runtime_for_user(&root, "user-alice"); // Owner
    let bob = runtime_for_user(&root, "user-bob");     // Member

    // Create channel with Alice (Owner) and Bob (Member)
    dispatch_ok(
        &alice,
        "createChannelConversation",
        json!({
            "channelId": "tech-broadcast",
            "title": "Tech Broadcast",
            "participantIds": ["user-bob"]
        }),
    );

    // Channel post test: Alice as Owner can post
    let msg_alice = dispatch_ok(
        &alice,
        "sendHumanMessage",
        json!({
            "conversationId": "channel-tech-broadcast",
            "text": "Broadcast from Alice",
            "clientNonce": "alice-msg-1"
        }),
    );
    assert_eq!(msg_alice["content"], "Broadcast from Alice");

    // Bob as regular channel member CANNOT post
    let err_bob = dispatch_err(
        &bob,
        "sendHumanMessage",
        json!({
            "conversationId": "channel-tech-broadcast",
            "text": "Bob attempting to post",
            "clientNonce": "bob-msg-1"
        }),
    );
    let err_str = err_bob.to_string();
    assert!(
        err_str.contains("ChannelPostNotAllowed"),
        "expected ChannelPostNotAllowed, got: {err_str}"
    );

    // Create group conversation
    dispatch_ok(
        &alice,
        "createGroupConversation",
        json!({
            "groupId": "dev-chat",
            "title": "Dev Chat",
            "participantIds": ["user-bob"]
        }),
    );

    // Bob can post in the group
    let bob_msg = dispatch_ok(
        &bob,
        "sendHumanMessage",
        json!({
            "conversationId": "group-dev-chat",
            "text": "Hello devs",
            "clientNonce": "bob-msg-2"
        }),
    );
    assert_eq!(bob_msg["content"], "Hello devs");

    // Restrict Bob from sending messages
    let mut restricted_bob = fabushi_messaging_core::CommunityMember {
        actor_id: fabushi_messaging_core::ActorId::new("user-bob"),
        status: fabushi_messaging_core::MemberStatus::Restricted,
        admin_title: None,
        admin_rights: fabushi_messaging_core::AdminRights::default(),
        restrictions: fabushi_messaging_core::MemberRestrictions {
            send_messages: true,
            ..Default::default()
        },
        joined_at_ms: 1000,
        invited_by: None,
    };
    dispatch_ok(
        &alice,
        "upsertConversationMember",
        json!({
            "conversationId": "group-dev-chat",
            "member": restricted_bob
        }),
    );

    // Bob now fails to send messages
    let err_restricted = dispatch_err(
        &bob,
        "sendHumanMessage",
        json!({
            "conversationId": "group-dev-chat",
            "text": "Bob trying to send while restricted",
            "clientNonce": "bob-msg-3"
        }),
    );
    assert!(err_restricted.to_string().contains("SendMessagesRestricted"));

    // Ban Bob
    restricted_bob.status = fabushi_messaging_core::MemberStatus::Banned;
    dispatch_ok(
        &alice,
        "upsertConversationMember",
        json!({
            "conversationId": "group-dev-chat",
            "member": restricted_bob
        }),
    );

    let err_banned = dispatch_err(
        &bob,
        "sendHumanMessage",
        json!({
            "conversationId": "group-dev-chat",
            "text": "Bob trying to send while banned",
            "clientNonce": "bob-msg-4"
        }),
    );
    assert!(
        err_banned.to_string().contains("MemberBanned")
            || err_banned.to_string().contains("not a conversation participant")
    );
}

#[test]
fn slow_mode_cooldown_and_owner_exemption() {
    let root = temp_root("slow-mode");
    let alice = runtime_for_user(&root, "user-alice"); // Owner
    let bob = runtime_for_user(&root, "user-bob");     // Member

    // Create group with 10s slow mode
    dispatch_ok(
        &alice,
        "createGroupConversation",
        json!({
            "groupId": "slow-group",
            "title": "Slow Mode Room",
            "participantIds": ["user-bob"],
            "slowModeSeconds": 10
        }),
    );

    // Bob sends message 1 -> succeeds
    let bob_msg1 = dispatch_ok(
        &bob,
        "sendHumanMessage",
        json!({
            "conversationId": "group-slow-group",
            "text": "First message from Bob",
            "clientNonce": "bob-slow-1"
        }),
    );
    assert_eq!(bob_msg1["content"], "First message from Bob");

    // Bob immediately attempts to send message 2 -> rejected by SlowModeActive
    let err_slow = dispatch_err(
        &bob,
        "sendHumanMessage",
        json!({
            "conversationId": "group-slow-group",
            "text": "Second message from Bob immediately",
            "clientNonce": "bob-slow-2"
        }),
    );
    let err_str = err_slow.to_string();
    assert!(
        err_str.contains("SlowModeActive"),
        "expected SlowModeActive, got: {err_str}"
    );

    // Alice as Owner is exempt: can send multiple messages consecutively
    dispatch_ok(
        &alice,
        "sendHumanMessage",
        json!({
            "conversationId": "group-slow-group",
            "text": "Alice 1",
            "clientNonce": "alice-slow-1"
        }),
    );
    dispatch_ok(
        &alice,
        "sendHumanMessage",
        json!({
            "conversationId": "group-slow-group",
            "text": "Alice 2",
            "clientNonce": "alice-slow-2"
        }),
    );
}

#[test]
fn forum_topic_lifecycle_and_administrative_authorization() {
    let root = temp_root("forum-topics");
    let alice = runtime_for_user(&root, "user-alice"); // Owner
    let bob = runtime_for_user(&root, "user-bob");     // Member

    dispatch_ok(
        &alice,
        "createGroupConversation",
        json!({
            "groupId": "forum-group",
            "title": "Forum Group",
            "participantIds": ["user-bob"]
        }),
    );

    // Non-admin (Bob) attempting to create topic is rejected
    let err_non_admin = dispatch_err(
        &bob,
        "upsertConversationTopic",
        json!({
            "conversationId": "group-forum-group",
            "topicId": "topic-general",
            "title": "General Discussion"
        }),
    );
    assert!(err_non_admin.to_string().contains("manage_topics"));

    // Owner (Alice) creates topic-general
    let topic_created = dispatch_ok(
        &alice,
        "upsertConversationTopic",
        json!({
            "conversationId": "group-forum-group",
            "topicId": "topic-general",
            "title": "General Discussion",
            "icon": "💬",
            "pinned": true
        }),
    );
    assert_eq!(topic_created["id"], "topic-general");
    assert_eq!(topic_created["title"], "General Discussion");
    assert_eq!(topic_created["pinned"], true);
    assert_eq!(topic_created["closed"], false);

    // List topics
    let topics = dispatch_ok(
        &alice,
        "listConversationTopics",
        json!({ "conversationId": "group-forum-group" }),
    );
    let topic_list = topics.as_array().expect("topics array");
    assert_eq!(topic_list.len(), 1);
    assert_eq!(topic_list[0]["id"], "topic-general");

    // Bob can send to open topic-general
    let msg_topic = dispatch_ok(
        &bob,
        "sendHumanMessage",
        json!({
            "conversationId": "group-forum-group",
            "text": "Bob in general topic",
            "clientNonce": "bob-topic-1",
            "topicId": "topic-general"
        }),
    );
    assert_eq!(msg_topic["topicId"], "topic-general");

    // Alice closes the topic
    dispatch_ok(
        &alice,
        "upsertConversationTopic",
        json!({
            "conversationId": "group-forum-group",
            "topicId": "topic-general",
            "title": "General Discussion",
            "closed": true
        }),
    );

    // Bob cannot send to closed topic
    let err_closed = dispatch_err(
        &bob,
        "sendHumanMessage",
        json!({
            "conversationId": "group-forum-group",
            "text": "Bob trying in closed topic",
            "clientNonce": "bob-topic-2",
            "topicId": "topic-general"
        }),
    );
    assert!(err_closed.to_string().contains("TopicClosed"));

    // Alice as Owner CAN post in closed topic
    let alice_closed_msg = dispatch_ok(
        &alice,
        "sendHumanMessage",
        json!({
            "conversationId": "group-forum-group",
            "text": "Alice posting in closed topic announcement",
            "clientNonce": "alice-topic-closed-1",
            "topicId": "topic-general"
        }),
    );
    assert_eq!(alice_closed_msg["topicId"], "topic-general");
}

#[test]
fn forward_to_topic_destination_contract() {
    let root = temp_root("forward-topic");
    let alice = runtime_for_user(&root, "user-alice");

    // Setup source conversation with a message
    let direct = dispatch_ok(
        &alice,
        "createHumanConversation",
        json!({
            "peerHumanId": "user-bob",
            "title": "Alice and Bob Direct"
        }),
    );
    let source_conv_id = direct["id"].as_str().expect("source conv id");
    let sent_source = dispatch_ok(
        &alice,
        "sendHumanMessage",
        json!({
            "conversationId": source_conv_id,
            "text": "Original message to forward",
            "clientNonce": "source-nonce-1"
        }),
    );
    let source_entry_id = sent_source["id"].as_str().expect("source entry id");

    // Setup destination group with a topic
    dispatch_ok(
        &alice,
        "createGroupConversation",
        json!({
            "groupId": "target-group",
            "title": "Target Group",
            "participantIds": ["user-bob"]
        }),
    );
    dispatch_ok(
        &alice,
        "upsertConversationTopic",
        json!({
            "conversationId": "group-target-group",
            "topicId": "topic-incoming",
            "title": "Incoming Shares"
        }),
    );

    // Forward to destination group and specify destinationTopics mapping + comment
    let forward_res = dispatch_ok(
        &alice,
        "forwardHumanMessage",
        json!({
            "sourceConversationId": source_conv_id,
            "sourceEntryId": source_entry_id,
            "destinationConversationIds": ["group-target-group"],
            "destinationTopics": {
                "group-target-group": "topic-incoming"
            },
            "clientNonce": "fwd-nonce-1",
            "comment": "Check this out!"
        }),
    );
    let destinations = forward_res["destinations"].as_array().expect("destinations");
    assert_eq!(destinations.len(), 1);
    assert_eq!(destinations[0]["status"], "sent");

    // Verify transcript in target group: comment and forwarded message both stamped with topic-incoming
    let owner = alice
        .open_human_conversation_db_owner("group-target-group")
        .expect("target db owner");
    let entries = owner.get_transcript_entries().expect("transcript");
    assert_eq!(entries.len(), 2);

    let comment_entry = &entries[0];
    assert_eq!(comment_entry["content"], "Check this out!");
    assert_eq!(comment_entry["topicId"], "topic-incoming");

    let fwd_entry = &entries[1];
    assert_eq!(fwd_entry["content"], "Original message to forward");
    assert_eq!(fwd_entry["topicId"], "topic-incoming");
}

#[test]
fn topic_destination_eligibility_fail_closed_contract() {
    let root = temp_root("topic-eligibility");
    let alice = runtime_for_user(&root, "user-alice"); // Owner
    let bob = runtime_for_user(&root, "user-bob");     // Regular member in target group

    // Alice creates target group
    dispatch_ok(
        &alice,
        "createGroupConversation",
        json!({
            "groupId": "target-community",
            "title": "Target Community",
            "participantIds": ["user-bob"]
        }),
    );

    // Create a closed topic
    dispatch_ok(
        &alice,
        "upsertConversationTopic",
        json!({
            "conversationId": "group-target-community",
            "topicId": "topic-closed",
            "title": "Closed Topic",
            "closed": true
        }),
    );

    // Create a hidden topic
    dispatch_ok(
        &alice,
        "upsertConversationTopic",
        json!({
            "conversationId": "group-target-community",
            "topicId": "topic-hidden",
            "title": "Hidden Topic",
            "hidden": true
        }),
    );

    // Bob creates source message in direct conversation with Alice
    let bob_source = dispatch_ok(
        &bob,
        "createHumanConversation",
        json!({
            "peerHumanId": "user-alice",
            "title": "Bob and Alice Direct"
        }),
    );
    assert_eq!(bob_source["kind"], "human");
    let bob_source_conv_id = bob_source["id"].as_str().expect("bob source conv id");
    let bob_msg = dispatch_ok(
        &bob,
        "sendHumanMessage",
        json!({
            "conversationId": bob_source_conv_id,
            "text": "Bob's source message",
            "clientNonce": "bob-elig-src-1"
        }),
    );
    let bob_source_entry_id = bob_msg["id"].as_str().expect("bob source id");

    // 1. Bob forwards to non-existent topic -> fails closed (TopicNotFound)
    let fwd_missing = dispatch_ok(
        &bob,
        "forwardHumanMessage",
        json!({
            "sourceConversationId": bob_source_conv_id,
            "sourceEntryId": bob_source_entry_id,
            "destinationConversationIds": ["group-target-community"],
            "destinationTopics": {
                "group-target-community": "topic-does-not-exist"
            },
            "clientNonce": "fwd-missing-1"
        }),
    );
    let dest_missing = &fwd_missing["destinations"][0];
    assert_eq!(dest_missing["status"], "failed");
    assert!(
        dest_missing["error"]
            .as_str()
            .unwrap_or_default()
            .contains("TopicNotFound"),
        "error should contain TopicNotFound: {:?}",
        dest_missing["error"]
    );

    // 2. Bob forwards to closed topic -> fails closed (TopicClosed)
    let fwd_closed = dispatch_ok(
        &bob,
        "forwardHumanMessage",
        json!({
            "sourceConversationId": bob_source_conv_id,
            "sourceEntryId": bob_source_entry_id,
            "destinationConversationIds": ["group-target-community"],
            "destinationTopics": {
                "group-target-community": "topic-closed"
            },
            "clientNonce": "fwd-closed-1"
        }),
    );
    let dest_closed = &fwd_closed["destinations"][0];
    assert_eq!(dest_closed["status"], "failed");
    assert!(
        dest_closed["error"]
            .as_str()
            .unwrap_or_default()
            .contains("TopicClosed"),
        "error should contain TopicClosed: {:?}",
        dest_closed["error"]
    );

    // 3. Bob forwards to hidden topic -> fails closed (TopicNotFound)
    let fwd_hidden = dispatch_ok(
        &bob,
        "forwardHumanMessage",
        json!({
            "sourceConversationId": bob_source_conv_id,
            "sourceEntryId": bob_source_entry_id,
            "destinationConversationIds": ["group-target-community"],
            "destinationTopics": {
                "group-target-community": "topic-hidden"
            },
            "clientNonce": "fwd-hidden-1"
        }),
    );
    let dest_hidden = &fwd_hidden["destinations"][0];
    assert_eq!(dest_hidden["status"], "failed");
    assert!(
        dest_hidden["error"]
            .as_str()
            .unwrap_or_default()
            .contains("TopicNotFound"),
        "error should contain TopicNotFound: {:?}",
        dest_hidden["error"]
    );

    // 4. Verify target group transcript has NOT been mutated by any of the failed forwards!
    let target_owner = alice
        .open_human_conversation_db_owner("group-target-community")
        .expect("target db owner");
    let entries = target_owner.get_transcript_entries().expect("transcript");
    assert!(
        entries.is_empty(),
        "fail-closed policy must not leave any transcript rows on failed topic destinations!"
    );
}
