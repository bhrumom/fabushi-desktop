use crate::storage::agent_paths::get_sand_agents_root_dir;

use super::extension::{HostMemoryExtension, start_memory_extension};

pub fn start_production_memory_extension() -> HostMemoryExtension {
    start_memory_extension(get_sand_agents_root_dir(None))
}
