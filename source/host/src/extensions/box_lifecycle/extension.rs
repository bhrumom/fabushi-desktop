use std::rc::Rc;

use crate::extensions::extension_ids_generated::HostExtensionId;

use super::box_lifecycle_service::BoxLifecycleService;

pub const BOX_LIFECYCLE_DEPENDENCIES: &[HostExtensionId] = &[HostExtensionId::Auth];

pub fn box_lifecycle_extension_id() -> HostExtensionId {
    HostExtensionId::BoxLifecycle
}

pub trait BoxLifecycleClientFactory<Auth> {
    type Client;

    fn create_sand_cursor_backend_client(&self, auth: Rc<Auth>) -> Self::Client;
}

pub fn start_box_lifecycle_extension<Auth, Factory>(
    auth: Rc<Auth>,
    factory: &Factory,
) -> BoxLifecycleService<Factory::Client>
where
    Factory: BoxLifecycleClientFactory<Auth>,
{
    BoxLifecycleService::new(factory.create_sand_cursor_backend_client(auth))
}
