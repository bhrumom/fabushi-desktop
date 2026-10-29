use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_MAX_CATALOG_BYTES: usize = 256 * 1024;

/// A bounded, side-effect-free parser for explicitly configured worker
/// provider discovery output. It never turns an advertised capability into
/// authorization or a launch decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloudWorkerCatalogParser {
    max_bytes: usize,
}

impl Default for CloudWorkerCatalogParser {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_MAX_CATALOG_BYTES,
        }
    }
}

impl CloudWorkerCatalogParser {
    pub fn with_max_bytes(max_bytes: usize) -> Result<Self, CloudWorkerCatalogError> {
        if max_bytes == 0 {
            return Err(CloudWorkerCatalogError::InvalidLimit);
        }
        Ok(Self { max_bytes })
    }

    pub fn parse(&self, output: &[u8]) -> Result<CloudWorkerCatalog, CloudWorkerCatalogError> {
        if output.len() > self.max_bytes {
            return Err(CloudWorkerCatalogError::OutputTooLarge {
                bytes: output.len(),
                max_bytes: self.max_bytes,
            });
        }
        let catalog: CloudWorkerCatalog =
            serde_json::from_slice(output).map_err(CloudWorkerCatalogError::MalformedJson)?;
        catalog.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerCatalog {
    pub providers: Vec<CloudWorkerProvider>,
}

impl CloudWorkerCatalog {
    pub fn validate(mut self) -> Result<Self, CloudWorkerCatalogError> {
        if self.providers.is_empty() {
            return Err(CloudWorkerCatalogError::EmptyCatalog);
        }

        let mut provider_ids = HashSet::new();
        for provider in &mut self.providers {
            provider.normalize_and_validate()?;
            if !provider_ids.insert(provider.id.clone()) {
                return Err(CloudWorkerCatalogError::DuplicateProvider(provider.id.clone()));
            }
        }
        self.providers.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerProvider {
    pub id: String,
    pub profiles: Vec<CloudWorkerProfile>,
}

impl CloudWorkerProvider {
    fn normalize_and_validate(&mut self) -> Result<(), CloudWorkerCatalogError> {
        normalize_id(&mut self.id).map_err(|_| CloudWorkerCatalogError::InvalidProviderId)?;
        if self.profiles.is_empty() {
            return Err(CloudWorkerCatalogError::EmptyProviderProfiles(self.id.clone()));
        }

        let mut profile_ids = HashSet::new();
        for profile in &mut self.profiles {
            profile.normalize_and_validate()?;
            if !profile_ids.insert(profile.id.clone()) {
                return Err(CloudWorkerCatalogError::DuplicateProfile {
                    provider: self.id.clone(),
                    profile: profile.id.clone(),
                });
            }
        }
        self.profiles.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerProfile {
    pub id: String,
    pub machines: Vec<CloudWorkerMachine>,
}

impl CloudWorkerProfile {
    fn normalize_and_validate(&mut self) -> Result<(), CloudWorkerCatalogError> {
        normalize_id(&mut self.id).map_err(|_| CloudWorkerCatalogError::InvalidProfileId)?;
        if self.machines.is_empty() {
            return Err(CloudWorkerCatalogError::EmptyProfileMachines(self.id.clone()));
        }
        for machine in &self.machines {
            machine.validate()?;
        }
        self.machines.sort_by(|left, right| {
            left.os
                .cmp(&right.os)
                .then(left.arch.cmp(&right.arch))
                .then(left.resources.cmp(&right.resources))
        });
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerMachine {
    pub os: CloudWorkerOs,
    pub arch: String,
    pub resources: CloudWorkerResourceLimits,
}

impl CloudWorkerMachine {
    fn validate(&self) -> Result<(), CloudWorkerCatalogError> {
        if self.arch.trim().is_empty() {
            return Err(CloudWorkerCatalogError::InvalidArchitecture);
        }
        if self.resources.cpu_cores == 0 || self.resources.memory_bytes == 0 {
            return Err(CloudWorkerCatalogError::InvalidResources);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CloudWorkerOs {
    Linux,
    Macos,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CloudWorkerResourceLimits {
    pub cpu_cores: u16,
    pub memory_bytes: u64,
}

#[derive(Debug, Error)]
pub enum CloudWorkerCatalogError {
    #[error("catalog parser limit must be greater than zero")]
    InvalidLimit,
    #[error("catalog output is {bytes} bytes, above the {max_bytes}-byte limit")]
    OutputTooLarge { bytes: usize, max_bytes: usize },
    #[error("catalog output is not valid JSON: {0}")]
    MalformedJson(#[source] serde_json::Error),
    #[error("catalog must contain at least one provider")]
    EmptyCatalog,
    #[error("provider id is empty or contains surrounding whitespace")]
    InvalidProviderId,
    #[error("provider {0} has no profiles")]
    EmptyProviderProfiles(String),
    #[error("provider id is duplicated: {0}")]
    DuplicateProvider(String),
    #[error("profile id is empty or contains surrounding whitespace")]
    InvalidProfileId,
    #[error("profile {0} has no machine choices")]
    EmptyProfileMachines(String),
    #[error("profile {profile} is duplicated under provider {provider}")]
    DuplicateProfile { provider: String, profile: String },
    #[error("machine architecture is empty")]
    InvalidArchitecture,
    #[error("machine resources must declare non-zero CPU and memory")]
    InvalidResources,
}

fn normalize_id(value: &mut String) -> Result<(), ()> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed != value {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &[u8] = br#"{
        "providers": [{
            "id": "local",
            "profiles": [{
                "id": "default",
                "machines": [{
                    "os": "Linux",
                    "arch": "x86_64",
                    "resources": {"cpu_cores": 2, "memory_bytes": 1073741824}
                }]
            }]
        }]
    }"#;

    #[test]
    fn parses_and_sorts_a_valid_catalog() {
        let catalog = CloudWorkerCatalogParser::default().parse(VALID).unwrap();
        assert_eq!(catalog.providers[0].id, "local");
        assert_eq!(catalog.providers[0].profiles[0].id, "default");
    }

    #[test]
    fn rejects_catalogs_over_the_bound() {
        let parser = CloudWorkerCatalogParser::with_max_bytes(1).unwrap();
        assert!(matches!(
            parser.parse(VALID),
            Err(CloudWorkerCatalogError::OutputTooLarge { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_provider_ids() {
        let input = br#"{"providers":[
            {"id":"same","profiles":[{"id":"a","machines":[{"os":"Linux","arch":"x86_64","resources":{"cpu_cores":1,"memory_bytes":1}}]}]},
            {"id":"same","profiles":[{"id":"b","machines":[{"os":"Linux","arch":"x86_64","resources":{"cpu_cores":1,"memory_bytes":1}}]}]}
        ]}"#;
        assert!(matches!(
            CloudWorkerCatalogParser::default().parse(input),
            Err(CloudWorkerCatalogError::DuplicateProvider(id)) if id == "same"
        ));
    }
}
