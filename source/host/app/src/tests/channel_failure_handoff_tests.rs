//! Causal controls for the shared shipping drain and channel-failure owner.

use super::super::{
    BackgroundWakes, ChannelFailureAction, ChannelFailureState,
    drain_channel_revival_batches, drive_channel_failure_work,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(10);

struct ReturningWorker {
    resume: mpsc::Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for ReturningWorker {
    fn drop(&mut self) {
        let _ = self.resume.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn failure(operation: &str) -> Value {
    json!({
        "addressToken": "slack:handoff-room",
        "deliveryOperationId": operation,
        "markedAtMs": 123.0,
        "sessionKey": "agent:handoff-agent",
        "reason": "delivery refused",
    })
}

#[test]
fn channel_failure_idle_predecessor_preserves_successor_and_exact_batch() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
    assert!(BackgroundWakes::<Value>::begin_revival(
        &mut wakes.lock().unwrap().reviving_channel_failure_agent_ids,
        "handoff-agent",
    ));
    let (idle_tx, idle_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let (returned_tx, returned_rx) = mpsc::channel();
    let predecessor_wakes = Arc::clone(&wakes);
    let predecessor = thread::spawn(move || {
        drain_channel_revival_batches(
            || {
                let batch = predecessor_wakes.lock().unwrap()
                    .take_channel_failure_work_or_end_revival("handoff-agent");
                assert!(batch.is_none());
                // The actual owner has released idle custody; hold the same
                // production drain before it can observe None and return.
                idle_tx.send(()).unwrap();
                resume_rx.recv_timeout(WAIT).expect("release the idle predecessor");
                batch
            },
            |_| panic!("an idle predecessor must not process a failure"),
            || BackgroundWakes::<Value>::end_revival(
                &mut predecessor_wakes.lock().unwrap().reviving_channel_failure_agent_ids,
                "handoff-agent",
            ),
        );
        returned_tx.send(()).unwrap();
    });
    let mut owned = ReturningWorker { resume: resume_tx, thread: Some(predecessor) };
    idle_rx.recv_timeout(WAIT).expect("atomic failure idle release");
    let expected = vec![failure("first-operation"), failure("second-operation")];
    let successor = {
        let mut state = wakes.lock().unwrap();
        let successor = state.enqueue_channel_failure_and_claim(
            "handoff-agent", expected[0].clone(),
        ).unwrap().expect("the first new failure admits one successor");
        assert!(state.enqueue_channel_failure_and_claim(
            "handoff-agent", expected[1].clone(),
        ).unwrap().is_none(), "later input cannot create a second worker");
        successor
    };
    owned.resume.send(()).unwrap();
    returned_rx.recv_timeout(WAIT).expect("predecessor returned through the shipping drain");
    owned.thread.take().unwrap().join().expect("owned predecessor");
    assert!(wakes.lock().unwrap().reviving_channel_failure_agent_ids.contains("handoff-agent"),
        "the returning predecessor must not retire its successor");

    let mut initial = Some(successor);
    let mut claimed = Vec::new();
    let mut claim_ids = std::collections::HashSet::new();
    drain_channel_revival_batches(
        || initial.take().or_else(|| wakes.lock().unwrap()
            .take_channel_failure_work_or_end_revival("handoff-agent")),
        |work| {
            assert!(wakes.try_lock().is_ok(), "processing runs outside the queue mutex");
            assert!(claim_ids.insert(work.claim_id.clone()), "each queued head is acquired once");
            claimed.push(work.failures.clone());
            assert!(wakes.lock().unwrap().finish_channel_failure_work("handoff-agent", &work));
            true
        },
        || panic!("idle release needs no second cleanup"),
    );
    assert_eq!(claimed, vec![vec![expected[0].clone()], vec![expected[1].clone()]],
        "input arriving after reservation remains in its ordered successor head");
    assert_eq!(claimed.into_iter().flatten().collect::<Vec<_>>(), expected);
    let state = wakes.lock().unwrap();
    assert!(!state.pending_channel_failures.contains_key("handoff-agent"));
    assert!(state.retained_channel_failures("handoff-agent").is_none());
    assert!(!state.reviving_channel_failure_agent_ids.contains("handoff-agent"));
}

#[test]
fn channel_failure_nonempty_pause_preserves_order_and_releases_once() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
    let first = failure("original-operation");
    let later = failure("later-operation");
    let initial = wakes.lock().unwrap().enqueue_channel_failure_and_claim(
        "handoff-agent", first.clone(),
    ).unwrap().unwrap();
    let initial_claim = initial.claim_id.clone();
    let initial_token = initial.worker_token.clone();
    let mut initial = Some(initial);
    let mut processed = 0;
    let mut stopped = 0;
    drain_channel_revival_batches(
        || initial.take().or_else(|| wakes.lock().unwrap()
            .take_channel_failure_work_or_end_revival("handoff-agent")),
        |work| drive_channel_failure_work(work, |work| {
            processed += 1;
            assert!(processed <= 2, "one coalesced signal cannot spin or replay indefinitely");
            assert_eq!(work.claim_id, initial_claim, "recheck retains the original head");
            assert_eq!(work.failures, vec![first.clone()]);
            {
                let mut state = wakes.try_lock().expect("processing is outside the queue mutex");
                assert!(state.reviving_channel_failure_agent_ids.contains("handoff-agent"));
                if processed == 1 {
                    assert_eq!(work.worker_token, initial_token);
                    assert!(state.enqueue_channel_failure_and_claim(
                        "handoff-agent", later.clone(),
                    ).unwrap().is_none(), "an active head only latches the new input signal");
                } else {
                    assert_ne!(work.worker_token, initial_token, "recheck owns a fresh token");
                }
            }
            let action = wakes.lock().unwrap().pause_channel_failure_before_dispatch(
                "handoff-agent", work, work.failures.clone(),
            );
            if processed == 1 {
                assert!(matches!(&action, ChannelFailureAction::Retry(_)));
            } else {
                assert!(matches!(&action, ChannelFailureAction::Held));
            }
            action
        }),
        || { stopped += 1; },
    );
    assert_eq!((processed, stopped), (2, 1),
        "the sole signal authorizes one recheck, then token retirement stops the drain once");
    let mut state = wakes.lock().unwrap();
    assert_eq!(state.channel_failure_state("handoff-agent"),
        Some(ChannelFailureState::PausedBeforeDispatch));
    assert_eq!(state.retained_channel_failures("handoff-agent"), Some([first.clone()].as_slice()));
    assert_eq!(state.pending_channel_failures.get("handoff-agent"), Some(&vec![later.clone()]));
    let retained_and_pending = state.retained_channel_failures("handoff-agent").unwrap().iter()
        .chain(state.pending_channel_failures.get("handoff-agent").unwrap().iter())
        .cloned().collect::<Vec<_>>();
    assert_eq!(retained_and_pending, vec![first.clone(), later]);
    assert!(!state.reviving_channel_failure_agent_ids.contains("handoff-agent"));
    let tickets = state.request_channel_failure_readiness();
    assert_eq!(tickets.len(), 1, "a later supported signal can claim the retained head once");
    assert_eq!(tickets[0].0, "handoff-agent");
    assert_eq!(tickets[0].1.claim_id, initial_claim);
    assert_eq!(tickets[0].1.failures, vec![first]);
}
