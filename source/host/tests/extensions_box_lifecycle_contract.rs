use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use mahayana_host_runtime::extensions::box_lifecycle::{
    BOX_LIFECYCLE_DEPENDENCIES, BoxLifecycleClient, BoxLifecycleClientFactory,
    BoxLifecycleFuture, BoxRunState, RecreateSandBoxRequest, RecreateSandBoxResponse,
    box_lifecycle_extension_id, start_box_lifecycle_extension,
};
use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;

struct NoopWake;

impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

fn block_on_ready<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("test future unexpectedly pending"),
    }
}

#[derive(Default)]
struct FakeAuth;

#[derive(Default)]
struct FakeClient {
    observed_signal: RefCell<Option<u64>>,
    recreate_requests: RefCell<Vec<RecreateSandBoxRequest>>,
    image_update_available: Cell<bool>,
}

impl BoxLifecycleClient<u64> for Rc<FakeClient> {
    type Error = &'static str;

    fn get_sand_box_run_state<'a>(
        &'a self,
        signal: &'a u64,
    ) -> BoxLifecycleFuture<'a, Result<BoxRunState, Self::Error>> {
        self.observed_signal.replace(Some(*signal));
        let available = self.image_update_available.get();
        Box::pin(async move {
            Ok(BoxRunState {
                image_update_available: available,
            })
        })
    }

    fn recreate_sand_box<'a>(
        &'a self,
        request: RecreateSandBoxRequest,
    ) -> BoxLifecycleFuture<'a, Result<RecreateSandBoxResponse, Self::Error>> {
        self.recreate_requests.borrow_mut().push(request);
        Box::pin(async {
            Ok(RecreateSandBoxResponse {
                started: true,
                reason: Some("recreating".to_string()),
            })
        })
    }
}

struct FakeFactory {
    client: Rc<FakeClient>,
    calls: Cell<usize>,
}

impl BoxLifecycleClientFactory<FakeAuth> for FakeFactory {
    type Client = Rc<FakeClient>;

    fn create_sand_cursor_backend_client(&self, _auth: Rc<FakeAuth>) -> Self::Client {
        self.calls.set(self.calls.get() + 1);
        Rc::clone(&self.client)
    }
}

#[test]
fn box_lifecycle_service_and_extension_preserve_grok_contract() {
    assert_eq!(box_lifecycle_extension_id(), HostExtensionId::BoxLifecycle);
    assert_eq!(BOX_LIFECYCLE_DEPENDENCIES, &[HostExtensionId::Auth]);

    let client = Rc::new(FakeClient::default());
    client.image_update_available.set(true);
    let factory = FakeFactory {
        client: Rc::clone(&client),
        calls: Cell::new(0),
    };
    let service = start_box_lifecycle_extension(Rc::new(FakeAuth), &factory);
    assert_eq!(factory.calls.get(), 1);

    let signal = 42_u64;
    let available =
        block_on_ready(service.fetch_image_update_available(&signal)).expect("run state");
    assert!(available);
    assert_eq!(client.observed_signal.borrow().as_ref(), Some(&42));

    let default_force =
        block_on_ready(service.recreate_in_box::<u64>(true, None)).expect("recreate");
    assert_eq!(
        default_force,
        RecreateSandBoxResponse {
            started: true,
            reason: Some("recreating".to_string()),
        }
    );
    let forced =
        block_on_ready(service.recreate_in_box::<u64>(false, Some(true))).expect("forced recreate");
    assert!(forced.started);

    assert_eq!(
        client.recreate_requests.borrow().as_slice(),
        &[
            RecreateSandBoxRequest {
                preserve_data: true,
                force: false,
            },
            RecreateSandBoxRequest {
                preserve_data: false,
                force: true,
            },
        ]
    );
}
