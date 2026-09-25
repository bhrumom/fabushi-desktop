use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::extensions::box_store_sync::request_coalescer::{
    RequestCoalescer, RequestCoalescerError,
};

fn wait_until(predicate: impl Fn() -> bool) {
    for _ in 0..500 {
        if predicate() {
            return;
        }
        thread::sleep(Duration::from_millis(2));
    }
    panic!("condition did not become true");
}

#[test]
fn conflicting_keys_are_deferred_to_a_later_batch() {
    let calls = Arc::new(AtomicUsize::new(0));
    let release_first = Arc::new(AtomicBool::new(false));
    let batches = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));

    let calls_for_run = Arc::clone(&calls);
    let release_for_run = Arc::clone(&release_first);
    let batches_for_run = Arc::clone(&batches);
    let coalescer = RequestCoalescer::<String, String, String>::new(
        3,
        Arc::new(move |requests| {
            let call = calls_for_run.fetch_add(1, Ordering::SeqCst);
            batches_for_run
                .lock()
                .expect("record batch")
                .push(requests.clone());
            if call == 0 {
                while !release_for_run.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            Ok(requests
                .into_iter()
                .map(|request| format!("{request}-ok"))
                .collect())
        }),
        Some(Arc::new(|request: &String| {
            request.split_once(':').map(|(key, _)| key.to_string())
        })),
        None,
    );

    let first = {
        let coalescer = Arc::clone(&coalescer);
        thread::spawn(move || coalescer.request("seed:0".into()).expect("seed request"))
    };
    wait_until(|| calls.load(Ordering::SeqCst) == 1);

    let mut workers = Vec::new();
    for request in ["a:1", "a:2", "b:1"] {
        let coalescer = Arc::clone(&coalescer);
        workers.push(thread::spawn(move || {
            coalescer.request(request.into()).expect("queued request")
        }));
    }
    thread::sleep(Duration::from_millis(20));
    release_first.store(true, Ordering::Release);

    assert_eq!(first.join().expect("seed thread"), "seed:0-ok");
    let results = workers
        .into_iter()
        .map(|worker| worker.join().expect("worker thread"))
        .collect::<Vec<_>>();
    assert_eq!(results, vec!["a:1-ok", "a:2-ok", "b:1-ok"]);

    let batches = batches.lock().expect("read batches").clone();
    assert_eq!(batches[0], vec!["seed:0"]);
    assert!(batches.iter().any(|batch| batch == &vec!["a:1".to_string(), "b:1".to_string()]));
    assert!(batches.iter().any(|batch| batch == &vec!["a:2".to_string()]));
}

#[test]
fn split_on_error_retries_failed_batch_individually() {
    let calls = Arc::new(AtomicUsize::new(0));
    let release_first = Arc::new(AtomicBool::new(false));
    let calls_for_run = Arc::clone(&calls);
    let release_for_run = Arc::clone(&release_first);

    let coalescer = RequestCoalescer::<String, String, String>::new(
        4,
        Arc::new(move |requests| {
            let call = calls_for_run.fetch_add(1, Ordering::SeqCst);
            if call == 0 {
                while !release_for_run.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(1));
                }
            }
            if requests.len() > 1 {
                return Err("split-me".into());
            }
            Ok(requests)
        }),
        None,
        Some(Arc::new(|error| {
            matches!(error, RequestCoalescerError::Run(value) if value == "split-me")
        })),
    );

    let seed = {
        let coalescer = Arc::clone(&coalescer);
        thread::spawn(move || coalescer.request("seed".into()).expect("seed"))
    };
    wait_until(|| calls.load(Ordering::SeqCst) == 1);

    let left = {
        let coalescer = Arc::clone(&coalescer);
        thread::spawn(move || coalescer.request("left".into()).expect("left"))
    };
    let right = {
        let coalescer = Arc::clone(&coalescer);
        thread::spawn(move || coalescer.request("right".into()).expect("right"))
    };
    thread::sleep(Duration::from_millis(20));
    release_first.store(true, Ordering::Release);

    assert_eq!(seed.join().expect("seed thread"), "seed");
    assert_eq!(left.join().expect("left thread"), "left");
    assert_eq!(right.join().expect("right thread"), "right");
    assert!(calls.load(Ordering::SeqCst) >= 4);
}

#[test]
fn result_count_mismatch_is_a_protocol_error() {
    let coalescer = RequestCoalescer::<String, String, String>::new(
        1,
        Arc::new(|_| Ok(Vec::new())),
        None,
        None,
    );
    let error = coalescer.request("one".into()).expect_err("must fail closed");
    assert!(matches!(
        error,
        RequestCoalescerError::Protocol(message)
            if message.contains("0 results for 1 requests")
    ));
}
