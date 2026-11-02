//! Causal controls for the shared shipping drain and channel-failure owner.

use super::super::{
    BackgroundWakes, drain_channel_revival_batches, requeue_channel_failures,
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
                    .take_pending_channel_failures_or_end_revival("handoff-agent");
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
    {
        let mut state = wakes.lock().unwrap();
        for (index, item) in expected.iter().enumerate() {
            BackgroundWakes::enqueue(&mut state.pending_channel_failures, "handoff-agent", item.clone());
            assert_eq!(
                BackgroundWakes::<Value>::begin_revival(
                    &mut state.reviving_channel_failure_agent_ids, "handoff-agent",
                ),
                index == 0,
                "the first new failure admits exactly one successor",
            );
        }
    }
    owned.resume.send(()).unwrap();
    returned_rx.recv_timeout(WAIT).expect("predecessor returned through the shipping drain");
    owned.thread.take().unwrap().join().expect("owned predecessor");
    assert!(wakes.lock().unwrap().reviving_channel_failure_agent_ids.contains("handoff-agent"),
        "the returning predecessor must not retire its successor");

    let mut claimed = Vec::new();
    drain_channel_revival_batches(
        || wakes.lock().unwrap().take_pending_channel_failures_or_end_revival("handoff-agent"),
        |batch| {
            assert!(wakes.try_lock().is_ok(), "processing runs outside the queue mutex");
            claimed.push(batch);
            assert_eq!(claimed.len(), 1, "the queued failures are claimed once");
            true
        },
        || panic!("idle release needs no second cleanup"),
    );
    assert_eq!(claimed, vec![expected]);
    let state = wakes.lock().unwrap();
    assert!(!state.pending_channel_failures.contains_key("handoff-agent"));
    assert!(!state.reviving_channel_failure_agent_ids.contains("handoff-agent"));
}

#[test]
fn channel_failure_nonempty_pause_preserves_order_and_releases_once() {
    let wakes = Arc::new(Mutex::new(BackgroundWakes::<Value>::default()));
    let first = failure("original-operation");
    let later = failure("later-operation");
    {
        let mut state = wakes.lock().unwrap();
        BackgroundWakes::enqueue(&mut state.pending_channel_failures, "handoff-agent", first.clone());
        assert!(BackgroundWakes::<Value>::begin_revival(
            &mut state.reviving_channel_failure_agent_ids, "handoff-agent",
        ));
    }
    let mut processed = 0;
    let mut stopped = 0;
    drain_channel_revival_batches(
        || wakes.lock().unwrap().take_pending_channel_failures_or_end_revival("handoff-agent"),
        |batch| {
            processed += 1;
            assert_eq!(processed, 1, "a nonempty pause must not spin or replay");
            assert_eq!(batch, vec![first.clone()]);
            {
                let mut state = wakes.try_lock().expect("processing is outside the queue mutex");
                assert!(state.reviving_channel_failure_agent_ids.contains("handoff-agent"));
                BackgroundWakes::enqueue(&mut state.pending_channel_failures, "handoff-agent", later.clone());
                assert!(!BackgroundWakes::<Value>::begin_revival(
                    &mut state.reviving_channel_failure_agent_ids, "handoff-agent",
                ));
            }
            requeue_channel_failures(&wakes, "handoff-agent", batch);
            false
        },
        || {
            stopped += 1;
            BackgroundWakes::<Value>::end_revival(
                &mut wakes.lock().unwrap().reviving_channel_failure_agent_ids,
                "handoff-agent",
            );
        },
    );
    assert_eq!((processed, stopped), (1, 1));
    let mut state = wakes.lock().unwrap();
    assert_eq!(state.pending_channel_failures.get("handoff-agent"), Some(&vec![first, later]));
    assert!(!state.reviving_channel_failure_agent_ids.contains("handoff-agent"));
    assert!(BackgroundWakes::<Value>::begin_revival(
        &mut state.reviving_channel_failure_agent_ids, "handoff-agent",
    ), "a later existing signal can claim the retained batch");
}
