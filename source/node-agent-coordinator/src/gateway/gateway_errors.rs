#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GatewayError {
    #[error("gateway is unreachable: {0}")] Unreachable(String),
    #[error("gateway command failed ({status}): {message}")] Command { status: u16, message: String },
    #[error("gateway protocol error: {0}")] Protocol(String),
}
