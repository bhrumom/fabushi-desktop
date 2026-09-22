use std::collections::HashSet;

use serde_json::Value;

use crate::webauthn::{
    ApprovedWebAuthnConsent, WebAuthnCeremony, WebAuthnSigner, WebAuthnSignerError,
    WebAuthnSignerResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum WebAuthnRequestFrame {
    Welcome { provider_id: String },
    Ceremony {
        request_id: String,
        ceremony: WebAuthnCeremony,
    },
    Cancel { request_id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebAuthnResponseFrame {
    Hello {
        computer_id: Option<String>,
        label: Option<String>,
    },
    Stage {
        request_id: String,
        stage: &'static str,
        outcome: &'static str,
    },
    Result {
        request_id: String,
        credential_json: Value,
    },
    Error {
        request_id: String,
        error: WebAuthnSignerError,
    },
    Ping,
}

#[derive(Debug)]
pub struct WebAuthnProvider<S: WebAuthnSigner> {
    signer: S,
    computer_id: Option<String>,
    label: Option<String>,
    provider_id: Option<String>,
    in_flight: HashSet<String>,
}

impl<S: WebAuthnSigner> WebAuthnProvider<S> {
    pub fn new(
        signer: S,
        computer_id: Option<String>,
        label: Option<String>,
    ) -> Self {
        Self {
            signer,
            computer_id,
            label,
            provider_id: None,
            in_flight: HashSet::new(),
        }
    }

    pub fn provider_id(&self) -> Option<&str> {
        self.provider_id.as_deref()
    }

    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    pub fn reset_transport(&mut self) {
        self.provider_id = None;
        self.in_flight.clear();
    }

    pub fn heartbeat(&self) -> WebAuthnResponseFrame {
        WebAuthnResponseFrame::Ping
    }

    pub fn handle_frame(
        &mut self,
        frame: WebAuthnRequestFrame,
        consent: Option<ApprovedWebAuthnConsent>,
    ) -> Vec<WebAuthnResponseFrame> {
        match frame {
            WebAuthnRequestFrame::Welcome { provider_id } => {
                self.provider_id = Some(provider_id);
                vec![WebAuthnResponseFrame::Hello {
                    computer_id: self.computer_id.clone(),
                    label: self.label.clone(),
                }]
            }
            WebAuthnRequestFrame::Cancel { request_id } => {
                self.in_flight.remove(&request_id);
                Vec::new()
            }
            WebAuthnRequestFrame::Ceremony {
                request_id,
                ceremony,
            } => self.run_ceremony(request_id, ceremony, consent),
        }
    }

    fn run_ceremony(
        &mut self,
        request_id: String,
        ceremony: WebAuthnCeremony,
        consent: Option<ApprovedWebAuthnConsent>,
    ) -> Vec<WebAuthnResponseFrame> {
        if request_id.trim().is_empty() || !self.in_flight.insert(request_id.clone()) {
            return vec![WebAuthnResponseFrame::Error {
                request_id,
                error: WebAuthnSignerError {
                    name: "InvalidStateError".into(),
                    code: Some("duplicate_request".into()),
                    message: "WebAuthn request id is empty or already active".into(),
                },
            }];
        }

        if consent.as_ref().is_some_and(|value| !value.approved) {
            self.in_flight.remove(&request_id);
            return vec![
                WebAuthnResponseFrame::Stage {
                    request_id: request_id.clone(),
                    stage: "grant",
                    outcome: "declined",
                },
                WebAuthnResponseFrame::Error {
                    request_id,
                    error: WebAuthnSignerError {
                        name: "NotAllowedError".into(),
                        code: Some("consent_declined".into()),
                        message: "The security key request was declined on this computer".into(),
                    },
                },
            ];
        }

        let mut frames = Vec::new();
        if consent.is_some() {
            frames.push(WebAuthnResponseFrame::Stage {
                request_id: request_id.clone(),
                stage: "grant",
                outcome: "ok",
            });
        }

        match self.signer.sign(&ceremony, consent.as_ref()) {
            WebAuthnSignerResult::Success { credential_json } => {
                frames.push(WebAuthnResponseFrame::Stage {
                    request_id: request_id.clone(),
                    stage: "sign",
                    outcome: "ok",
                });
                frames.push(WebAuthnResponseFrame::Result {
                    request_id: request_id.clone(),
                    credential_json,
                });
            }
            WebAuthnSignerResult::Failed { error } => {
                frames.push(WebAuthnResponseFrame::Stage {
                    request_id: request_id.clone(),
                    stage: "sign",
                    outcome: "failed",
                });
                frames.push(WebAuthnResponseFrame::Error {
                    request_id: request_id.clone(),
                    error,
                });
            }
        }
        self.in_flight.remove(&request_id);
        frames
    }
}
