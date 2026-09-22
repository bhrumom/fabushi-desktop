use super::loopback_sand_box::{LoopbackSandBox, LoopbackSandBoxOptions};

pub fn create_sand_box(options: LoopbackSandBoxOptions) -> LoopbackSandBox {
    LoopbackSandBox::new(options)
}

pub fn format_sand_box_startup_summary(auto_update_enabled: bool, is_packaged: bool) -> String {
    format!(
        "[sand-host] agent box backend: loopback (in-box); image: host's own container; auto-update: {}; build: {}",
        if auto_update_enabled { "on" } else { "off" },
        if is_packaged { "packaged" } else { "dev" },
    )
}

pub fn should_apply_shared_desktop(max_windows: u32) -> bool {
    max_windows > 1
}
