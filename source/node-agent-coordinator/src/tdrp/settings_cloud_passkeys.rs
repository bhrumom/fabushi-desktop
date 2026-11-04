use crate::protocol::Failure;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum PrivacyRuleValue {
    AllowAll,
    ContactsOnly,
    DisallowAll,
    SpecificUsers {
        allowed: Vec<String>,
        disallowed: Vec<String>,
    },
}

impl Default for PrivacyRuleValue {
    fn default() -> Self {
        Self::AllowAll
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PrivacySettings {
    #[serde(default)]
    pub phone_number: PrivacyRuleValue,
    #[serde(default)]
    pub last_seen: PrivacyRuleValue,
    #[serde(default)]
    pub profile_photos: PrivacyRuleValue,
    #[serde(default)]
    pub forwards: PrivacyRuleValue,
    #[serde(default)]
    pub calls_p2p: PrivacyRuleValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudPasswordConfig {
    pub salt_hex: String,
    pub password_hash_hex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_email: Option<String>,
    pub attempts_left: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PasskeyRegistration {
    pub credential_id: String,
    pub public_key_base64: String,
    pub user_handle: String,
    pub created_at_ms: i64,
    pub counter: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SettingsCloudPasskeysState {
    pub privacy_settings: BTreeMap<String, PrivacySettings>,
    pub cloud_passwords: BTreeMap<String, CloudPasswordConfig>,
    pub passkeys: BTreeMap<String, Vec<PasskeyRegistration>>,
    pub pending_challenges: BTreeMap<String, (String, i64)>,
}

impl SettingsCloudPasskeysState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_privacy(&self, user_id: &str) -> PrivacySettings {
        self.privacy_settings
            .get(user_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn update_privacy(&mut self, user_id: &str, update: PrivacySettings) -> PrivacySettings {
        self.privacy_settings
            .insert(user_id.to_string(), update.clone());
        update
    }

    pub fn set_cloud_password(
        &mut self,
        user_id: &str,
        password_hash_hex: String,
        salt_hex: String,
        hint: Option<String>,
        recovery_email: Option<String>,
    ) -> Result<bool, Failure> {
        if password_hash_hex.is_empty() || salt_hex.is_empty() {
            return Err(Failure::new(
                "INVALID_CREDENTIALS",
                "password hash and salt cannot be empty",
            ));
        }

        let config = CloudPasswordConfig {
            salt_hex,
            password_hash_hex,
            hint,
            recovery_email,
            attempts_left: 5,
        };
        self.cloud_passwords.insert(user_id.to_string(), config);
        Ok(true)
    }

    pub fn verify_cloud_password(
        &mut self,
        user_id: &str,
        candidate_hash_hex: &str,
    ) -> Result<bool, Failure> {
        let config = self.cloud_passwords.get_mut(user_id).ok_or_else(|| {
            Failure::new(
                "PASSWORD_NOT_CONFIGURED",
                format!("no 2FA password set for {user_id}"),
            )
        })?;

        if config.attempts_left == 0 {
            return Err(Failure::new(
                "PASSWORD_LOCKED",
                "too many failed attempts, password verification locked",
            ));
        }

        if config.password_hash_hex == candidate_hash_hex {
            config.attempts_left = 5;
            Ok(true)
        } else {
            config.attempts_left = config.attempts_left.saturating_sub(1);
            Err(Failure::new(
                "PASSWORD_INCORRECT",
                format!("incorrect password; {} attempts left", config.attempts_left),
            ))
        }
    }

    pub fn remove_cloud_password(
        &mut self,
        user_id: &str,
        candidate_hash_hex: &str,
    ) -> Result<bool, Failure> {
        self.verify_cloud_password(user_id, candidate_hash_hex)?;
        self.cloud_passwords.remove(user_id);
        Ok(true)
    }

    pub fn create_auth_challenge(&mut self, user_id: &str, now_ms: i64) -> String {
        let challenge = format!("chal_{}_{}", user_id, now_ms);
        self.pending_challenges
            .insert(user_id.to_string(), (challenge.clone(), now_ms));
        challenge
    }

    pub fn register_passkey(
        &mut self,
        user_id: &str,
        credential_id: String,
        public_key_base64: String,
        challenge: &str,
        now_ms: i64,
    ) -> Result<PasskeyRegistration, Failure> {
        let pending = self
            .pending_challenges
            .get(user_id)
            .ok_or_else(|| Failure::new("NO_PENDING_CHALLENGE", "no active passkey challenge"))?;

        if pending.0 != challenge {
            return Err(Failure::new(
                "CHALLENGE_MISMATCH",
                "challenge does not match issued challenge",
            ));
        }
        self.pending_challenges.remove(user_id);

        let reg = PasskeyRegistration {
            credential_id: credential_id.clone(),
            public_key_base64,
            user_handle: user_id.to_string(),
            created_at_ms: now_ms,
            counter: 0,
        };
        self.passkeys
            .entry(user_id.to_string())
            .or_default()
            .push(reg.clone());
        Ok(reg)
    }

    pub fn verify_passkey(
        &mut self,
        user_id: &str,
        credential_id: &str,
        challenge: &str,
        signature_base64: &str,
    ) -> Result<bool, Failure> {
        if signature_base64.is_empty() {
            return Err(Failure::new(
                "INVALID_SIGNATURE",
                "passkey signature cannot be empty",
            ));
        }

        let pending = self
            .pending_challenges
            .get(user_id)
            .ok_or_else(|| Failure::new("NO_PENDING_CHALLENGE", "no active passkey challenge"))?;

        if pending.0 != challenge {
            return Err(Failure::new(
                "CHALLENGE_MISMATCH",
                "challenge does not match issued challenge",
            ));
        }
        self.pending_challenges.remove(user_id);

        let user_keys = self.passkeys.get_mut(user_id).ok_or_else(|| {
            Failure::new("PASSKEY_NOT_FOUND", "no passkeys registered for user")
        })?;

        let key = user_keys
            .iter_mut()
            .find(|k| k.credential_id == credential_id)
            .ok_or_else(|| Failure::new("CREDENTIAL_NOT_FOUND", "credential not found"))?;

        key.counter += 1;
        Ok(true)
    }

    pub fn handle_route(&mut self, method: &str, args: &Value) -> Result<Value, Failure> {
        if let Some(op) = method.strip_prefix("settings.") {
            match op {
                "getPrivacy" => {
                    let user_id = args
                        .get("userId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "userId required"))?;
                    let privacy = self.get_privacy(user_id);
                    Ok(json!({ "privacy": privacy }))
                }
                "updatePrivacy" => {
                    let user_id = args
                        .get("userId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "userId required"))?;
                    let update: PrivacySettings = args
                        .get("privacy")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    let res = self.update_privacy(user_id, update);
                    Ok(json!({ "privacy": res }))
                }
                other => Err(Failure::new(
                    "UNKNOWN_SETTINGS_METHOD",
                    format!("unrecognized settings method {other}"),
                )),
            }
        } else if let Some(op) = method.strip_prefix("auth.") {
            match op {
                "setCloudPassword" => {
                    let user_id = args
                        .get("userId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "userId required"))?;
                    let password_hash_hex = args
                        .get("passwordHashHex")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "passwordHashHex required"))?
                        .to_string();
                    let salt_hex = args
                        .get("saltHex")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "saltHex required"))?
                        .to_string();
                    let hint = args.get("hint").and_then(Value::as_str).map(ToString::to_string);
                    let email = args.get("recoveryEmail").and_then(Value::as_str).map(ToString::to_string);

                    let res = self.set_cloud_password(user_id, password_hash_hex, salt_hex, hint, email)?;
                    Ok(json!({ "success": res }))
                }
                "verifyCloudPassword" => {
                    let user_id = args
                        .get("userId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "userId required"))?;
                    let candidate = args
                        .get("passwordHashHex")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "passwordHashHex required"))?;

                    let res = self.verify_cloud_password(user_id, candidate)?;
                    Ok(json!({ "verified": res }))
                }
                "removeCloudPassword" => {
                    let user_id = args
                        .get("userId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "userId required"))?;
                    let candidate = args
                        .get("passwordHashHex")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "passwordHashHex required"))?;

                    let res = self.remove_cloud_password(user_id, candidate)?;
                    Ok(json!({ "removed": res }))
                }
                "createChallenge" => {
                    let user_id = args
                        .get("userId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "userId required"))?;
                    let now_ms = args
                        .get("nowMs")
                        .and_then(Value::as_i64)
                        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

                    let chal = self.create_auth_challenge(user_id, now_ms);
                    Ok(json!({ "challenge": chal }))
                }
                "registerPasskey" => {
                    let user_id = args
                        .get("userId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "userId required"))?;
                    let credential_id = args
                        .get("credentialId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "credentialId required"))?
                        .to_string();
                    let public_key_base64 = args
                        .get("publicKeyBase64")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "publicKeyBase64 required"))?
                        .to_string();
                    let challenge = args
                        .get("challenge")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "challenge required"))?;
                    let now_ms = args
                        .get("nowMs")
                        .and_then(Value::as_i64)
                        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

                    let reg = self.register_passkey(user_id, credential_id, public_key_base64, challenge, now_ms)?;
                    Ok(json!({ "registration": reg }))
                }
                "verifyPasskey" => {
                    let user_id = args
                        .get("userId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "userId required"))?;
                    let credential_id = args
                        .get("credentialId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "credentialId required"))?;
                    let challenge = args
                        .get("challenge")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "challenge required"))?;
                    let signature = args
                        .get("signatureBase64")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "signatureBase64 required"))?;

                    let verified = self.verify_passkey(user_id, credential_id, challenge, signature)?;
                    Ok(json!({ "verified": verified }))
                }
                other => Err(Failure::new(
                    "UNKNOWN_AUTH_METHOD",
                    format!("unrecognized auth method {other}"),
                )),
            }
        } else {
            Err(Failure::new(
                "UNKNOWN_METHOD",
                format!("unrecognized method {method}"),
            ))
        }
    }
}
