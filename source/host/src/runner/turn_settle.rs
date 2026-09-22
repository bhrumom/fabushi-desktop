#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalOutcome {
    Completed,
    Failed { retryable: bool, message: String },
    Cancelled,
    WaitingUser,
}

#[derive(Debug, Default)]
pub struct TurnSettlement {
    outcome: Option<TerminalOutcome>,
}

impl TurnSettlement {
    pub fn settle(&mut self, outcome: TerminalOutcome) -> Result<(), &'static str> {
        if self.outcome.is_some() {
            return Err("turn already has a terminal outcome");
        }
        self.outcome = Some(outcome);
        Ok(())
    }

    pub fn outcome(&self) -> Option<&TerminalOutcome> {
        self.outcome.as_ref()
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.outcome,
            Some(TerminalOutcome::Completed | TerminalOutcome::Failed { .. } | TerminalOutcome::Cancelled)
        )
    }
}
