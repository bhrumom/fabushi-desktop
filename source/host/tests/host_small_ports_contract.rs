use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use mahayana_host_runtime::r#box::box_env::{
    BoxEnvironmentControlClient, BoxEnvironmentUpdate, apply_box_environment_via_transport,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordedEnvironmentCall {
    ctx: String,
    update: BoxEnvironmentUpdate,
}

struct RecordingTransport {
    calls: Rc<RefCell<Vec<RecordedEnvironmentCall>>>,
}

struct RecordingControlClient {
    calls: Rc<RefCell<Vec<RecordedEnvironmentCall>>>,
}

impl BoxEnvironmentControlClient<String> for RecordingControlClient {
    type Error = &'static str;

    fn update_environment_variables(
        &mut self,
        ctx: &String,
        request: BoxEnvironmentUpdate,
    ) -> Result<(), Self::Error> {
        self.calls.borrow_mut().push(RecordedEnvironmentCall {
            ctx: ctx.clone(),
            update: request,
        });
        Ok(())
    }
}

#[test]
fn box_environment_port_is_wired_and_forwards_a_cloned_update() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let transport = RecordingTransport {
        calls: Rc::clone(&calls),
    };
    let ctx = "request-context".to_string();
    let update = BoxEnvironmentUpdate {
        env: BTreeMap::from([
            ("FABUSHI_AGENT".to_string(), "enabled".to_string()),
            ("SHELL".to_string(), "/bin/zsh".to_string()),
        ]),
        replace: true,
    };
    let original = update.clone();

    apply_box_environment_via_transport(&ctx, &transport, &update, |transport| {
        RecordingControlClient {
            calls: Rc::clone(&transport.calls),
        }
    })
    .expect("box environment transport should succeed");

    assert_eq!(update, original, "the caller-owned update must remain unchanged");
    assert_eq!(
        calls.borrow().as_slice(),
        &[RecordedEnvironmentCall {
            ctx,
            update: original,
        }]
    );
}
