//! Mahayana implementation of the Grok Bot 0.18 Node Agent Coordinator role.
//!
//! This crate intentionally owns coordinator state separately from both the
//! Electron process and the Mahayana Host/Runner.  It is a protocol and
//! supervision boundary, not a renderer helper.

pub mod carrier;
pub mod control_port_client;
pub mod gateway;
pub mod inference_router;
pub mod local_exec;
pub mod mcp;
pub mod oauth;
pub mod protocol;
pub mod renderer_port_server;
pub mod supervisor;
pub mod telemetry;
pub mod webauthn;

pub use protocol::{
    CoordinatorFrame, Failure, LifecyclePhase, ReplyOutcome, ShutdownReason,
    COORDINATOR_CANCELLED, COORDINATOR_PROTOCOL_VERSION,
};
pub use supervisor::{
    CoordinatorCrash, CoordinatorSupervisor, GatewayState, HostGeneration,
    PendingRequest, RequestSettlement, ResyncSnapshot,
};
