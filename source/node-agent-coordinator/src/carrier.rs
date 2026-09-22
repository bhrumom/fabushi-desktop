use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct CarrierEvent {
    pub family: String,
    pub payload: Value,
}

#[derive(Debug, Default)]
pub struct Carrier {
    queued: Vec<CarrierEvent>,
}

impl Carrier {
    pub fn publish(&mut self, family: impl Into<String>, payload: Value) {
        self.queued.push(CarrierEvent { family: family.into(), payload });
    }

    pub fn drain(&mut self) -> Vec<CarrierEvent> {
        std::mem::take(&mut self.queued)
    }
}
