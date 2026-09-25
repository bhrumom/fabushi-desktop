pub mod transcript;

#[path = "extension_ids.generated.rs"]
pub mod extension_ids_generated;
pub mod registry;
pub mod turn_execution;
pub mod local_exec;

pub mod session;
pub mod settings;
pub mod memory;
pub mod box_store_sync;
pub mod cloud_agents;
pub mod telemetry;
pub mod trays;
pub mod source_map;
pub mod webauthn_proxy;
pub mod browser_ua;
pub mod experiments;
pub mod box_lifecycle;
pub mod inference;

pub mod auth;

pub mod managed_setup;

pub mod forever_box;

pub mod cross_user_sharing;

pub mod wallpaper;

pub mod codebase_telemetry;

pub mod auto_review;

pub mod secrets;

pub mod state_backstop;

pub mod local_tool_permission;

pub mod action_audit;
