use std::collections::BTreeSet;

use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;
use mahayana_host_runtime::extensions::registry::HOST_EXTENSION_ORDER;
use mahayana_host_runtime::host_production_extensions::{
    CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS, ProductionBrowserUaLog,
    ProductionHostExtensions,
};

#[test]
fn current_shipping_subset_is_declared_in_the_frozen_35_slot_registry() {
    let all = HOST_EXTENSION_ORDER.iter().copied().collect::<BTreeSet<_>>();
    let shipping = CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        shipping.len(),
        CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS.len(),
        "shipping production extension ids must not contain duplicates"
    );
    assert!(
        shipping.iter().all(|id| all.contains(id)),
        "every shipping production extension must occupy a frozen Grok registry slot"
    );
    assert_eq!(HOST_EXTENSION_ORDER.len(), 35);
    assert!(shipping.contains(&HostExtensionId::Auth));
    assert!(shipping.contains(&HostExtensionId::BrowserUa));
    assert!(shipping.contains(&HostExtensionId::CloudAgents));
    assert!(shipping.contains(&HostExtensionId::WebauthnProxy));
}

#[test]
fn shipping_production_extension_owner_remains_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ProductionHostExtensions>();
    assert_send_sync::<ProductionBrowserUaLog>();
}
