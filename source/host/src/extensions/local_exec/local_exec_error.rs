#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct SandLocalExecError {
    pub message: String,
    outcome_unknown: bool,
}

impl SandLocalExecError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), outcome_unknown: false }
    }

    /// The provider may have entered execution, or an observed result could
    /// not be durably settled. This disposition must survive consumer mapping.
    pub fn outcome_unknown(message: impl Into<String>) -> Self {
        Self { message: message.into(), outcome_unknown: true }
    }

    pub fn is_outcome_unknown(&self) -> bool {
        self.outcome_unknown
    }
}
