use std::collections::BTreeMap;

use thiserror::Error;

const MAX_ENV_NAME_CHARS: usize = 128;
const MAX_ENV_VALUE_CHARS: usize = 4096;
const MAX_ENV_ENTRIES: usize = 64;
const MAX_POLICY_DIGITS: usize = 8;
const MIN_REFRESH_AFTER_MS: u64 = 3_600_000;
const MIN_RETAIN_UNUSED_MS: u64 = 86_400_000;

/// A deterministic, side-effect-free representation of the setup environment
/// profile. Values are inputs to a future secret-egress/process owner; this
/// type never writes a file or mutates a child environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudWorkerEnvironmentProfile {
    entries: BTreeMap<String, String>,
}

impl CloudWorkerEnvironmentProfile {
    pub fn new(
        entries: BTreeMap<String, String>,
    ) -> Result<Self, CloudWorkerAuxiliaryError> {
        if entries.len() > MAX_ENV_ENTRIES {
            return Err(CloudWorkerAuxiliaryError::EnvironmentEntryLimitReached);
        }
        for (name, value) in &entries {
            if !valid_environment_name(name) {
                return Err(CloudWorkerAuxiliaryError::InvalidEnvironmentName);
            }
            if value.chars().count() > MAX_ENV_VALUE_CHARS
                || value.chars().any(|character| character.is_control())
                || value.contains('`')
                || value.contains("$(")
            {
                return Err(CloudWorkerAuxiliaryError::UnsafeEnvironmentValue);
            }
        }
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &BTreeMap<String, String> {
        &self.entries
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }
}

/// Warm-image retention policy parsed from the provider configuration. It is
/// policy data only: allocation, capture, deletion and scheduling stay with
/// their existing owners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloudWorkerWarmImagePolicy {
    pub refresh_after_ms: u64,
    pub retain_unused_ms: u64,
    pub keep_previous: bool,
}

impl CloudWorkerWarmImagePolicy {
    pub fn from_config(
        refresh_after: Option<&str>,
        retain_unused: Option<&str>,
        keep_previous: bool,
    ) -> Result<Self, CloudWorkerAuxiliaryError> {
        let refresh_after_ms = parse_duration(
            refresh_after.unwrap_or("24h"),
            MIN_REFRESH_AFTER_MS,
            "refreshAfter",
        )?;
        let retain_unused_ms = parse_duration(
            retain_unused.unwrap_or("14d"),
            MIN_RETAIN_UNUSED_MS,
            "retainUnused",
        )?;
        Ok(Self {
            refresh_after_ms,
            retain_unused_ms,
            keep_previous,
        })
    }
}

fn parse_duration(
    value: &str,
    minimum_ms: u64,
    field: &'static str,
) -> Result<u64, CloudWorkerAuxiliaryError> {
    let Some(unit) = value.chars().last() else {
        return Err(CloudWorkerAuxiliaryError::InvalidDuration { field });
    };
    let digits = &value[..value.len().saturating_sub(unit.len_utf8())];
    if digits.is_empty()
        || digits.len() > MAX_POLICY_DIGITS
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || digits.starts_with('0')
    {
        return Err(CloudWorkerAuxiliaryError::InvalidDuration { field });
    }
    let multiplier = match unit {
        'm' => 60_000,
        'h' => 3_600_000,
        'd' => 86_400_000,
        _ => return Err(CloudWorkerAuxiliaryError::InvalidDuration { field }),
    };
    let amount = digits
        .parse::<u64>()
        .map_err(|_| CloudWorkerAuxiliaryError::InvalidDuration { field })?;
    let milliseconds = amount
        .checked_mul(multiplier)
        .ok_or(CloudWorkerAuxiliaryError::DurationOverflow { field })?;
    if milliseconds < minimum_ms {
        return Err(CloudWorkerAuxiliaryError::DurationTooShort { field });
    }
    Ok(milliseconds)
}

fn valid_environment_name(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_ENV_NAME_CHARS
        && !value.chars().any(|character| character.is_control() || character == '=')
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CloudWorkerAuxiliaryError {
    #[error("setup environment contains too many entries")]
    EnvironmentEntryLimitReached,
    #[error("setup environment name is empty, too long, or invalid")]
    InvalidEnvironmentName,
    #[error("setup environment value is unsafe or too long")]
    UnsafeEnvironmentValue,
    #[error("warm-image policy duration for {field} is invalid")]
    InvalidDuration { field: &'static str },
    #[error("warm-image policy duration for {field} overflows")]
    DurationOverflow { field: &'static str },
    #[error("warm-image policy duration for {field} is below the minimum")]
    DurationTooShort { field: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_profile_is_sorted_and_rejects_shell_unsafe_values() {
        let mut entries = BTreeMap::new();
        entries.insert("Z_VAR".into(), "z".into());
        entries.insert("A_VAR".into(), "a".into());
        let profile = CloudWorkerEnvironmentProfile::new(entries).unwrap();
        assert_eq!(profile.names().collect::<Vec<_>>(), vec!["A_VAR", "Z_VAR"]);

        let mut unsafe_entries = BTreeMap::new();
        unsafe_entries.insert("SECRET".into(), "$(cat /secret)".into());
        assert_eq!(
            CloudWorkerEnvironmentProfile::new(unsafe_entries),
            Err(CloudWorkerAuxiliaryError::UnsafeEnvironmentValue)
        );
    }

    #[test]
    fn warm_image_policy_matches_pinned_defaults_and_minimums() {
        assert_eq!(
            CloudWorkerWarmImagePolicy::from_config(None, None, false).unwrap(),
            CloudWorkerWarmImagePolicy {
                refresh_after_ms: 24 * 3_600_000,
                retain_unused_ms: 14 * 86_400_000,
                keep_previous: false,
            }
        );
        assert_eq!(
            CloudWorkerWarmImagePolicy::from_config(Some("30m"), None, false),
            Err(CloudWorkerAuxiliaryError::DurationTooShort {
                field: "refreshAfter"
            })
        );
    }

    #[test]
    fn warm_image_policy_rejects_zero_prefixed_and_unbounded_durations() {
        assert!(matches!(
            CloudWorkerWarmImagePolicy::from_config(Some("01h"), None, false),
            Err(CloudWorkerAuxiliaryError::InvalidDuration { .. })
        ));
        assert!(matches!(
            CloudWorkerWarmImagePolicy::from_config(Some("99999999d"), None, false),
            Ok(CloudWorkerWarmImagePolicy { .. })
        ));
        assert!(matches!(
            CloudWorkerWarmImagePolicy::from_config(Some("100000000d"), None, false),
            Err(CloudWorkerAuxiliaryError::InvalidDuration { .. })
        ));
    }
}
