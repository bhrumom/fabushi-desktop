use mahayana_host_runtime::extensions::transcript::send_pipeline::{
    PersistedSendContext, RecoverySend,
};
use mahayana_host_runtime::extensions::transcript::turn_runtime::{
    QueuedTurnRecoveryCheck, should_supersede_stale_turn,
};
use mahayana_host_runtime::runner::conversation_state::{
    RecoveryUserMessage, would_recover_via_prepend,
};

fn message(id: &str, text: &str, confirmed: Option<bool>) -> RecoveryUserMessage {
    RecoveryUserMessage {
        id: id.into(),
        text: text.into(),
        confirmed,
    }
}

#[test]
fn frozen_runner_prepend_eligibility_uses_order_and_confirmed_flag() {
    let recent = vec![
        message("m1", "one", Some(true)),
        message("m2", "two", None),
        message("m3", "three", None),
    ];
    assert!(would_recover_via_prepend(&recent, "m3", "m2"));
    assert!(would_recover_via_prepend(&recent, "m2", "m2"));
    assert!(!would_recover_via_prepend(&recent, "m2", "m3"));
    assert!(!would_recover_via_prepend(&recent, "m3", "m1"));
    assert!(!would_recover_via_prepend(&recent, "missing", "m2"));
}

#[test]
fn turn_runtime_applies_all_frozen_stale_recovery_guards() {
    let old = PersistedSendContext {
        echo_entry_id: Some("m2".into()),
        user_message_id: Some("m2".into()),
        recent_user_messages: vec![
            message("m1", "one", None),
            message("m2", "two", None),
        ],
    };
    let latest = RecoverySend {
        epoch: 3,
        message_id: "m3".into(),
        recent_user_messages: vec![
            message("m1", "one", None),
            message("m2", "two", None),
            message("m3", "three", None),
        ],
    };
    let check = |break_epoch, is_fork, has_reply, attachments| {
        should_supersede_stale_turn(QueuedTurnRecoveryCheck {
            epoch: 2,
            current_epoch: 3,
            recovery_break_epoch: break_epoch,
            prompt: "two",
            context: &old,
            latest_recovery: Some(&latest),
            is_fork,
            has_reply_context: has_reply,
            attachment_count: attachments,
            image_count: 0,
            video_count: 0,
        })
    };
    assert!(check(0, false, false, 0));
    assert!(!check(2, false, false, 0));
    assert!(!check(0, true, false, 0));
    assert!(!check(0, false, true, 0));
    assert!(!check(0, false, false, 1));

    let confirmed_latest = RecoverySend {
        epoch: 3,
        message_id: "m3".into(),
        recent_user_messages: vec![
            message("m2", "two", Some(true)),
            message("m3", "three", None),
        ],
    };
    assert!(!should_supersede_stale_turn(QueuedTurnRecoveryCheck {
        epoch: 2,
        current_epoch: 3,
        recovery_break_epoch: 0,
        prompt: "two",
        context: &old,
        latest_recovery: Some(&confirmed_latest),
        is_fork: false,
        has_reply_context: false,
        attachment_count: 0,
        image_count: 0,
        video_count: 0,
    }));
}
