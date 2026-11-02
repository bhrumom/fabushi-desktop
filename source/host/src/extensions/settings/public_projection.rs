// Source-informed path and array-identity policy:
// openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845,
// src/config/{sensitive-paths,redact-snapshot,env-substitution}.ts and
// packages/net-policy/src/redact-sensitive-url.ts.
//
// Copyright (c) 2026 OpenClaw Foundation
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use percent_encoding::percent_decode_str;
use regex::Regex;
use serde_json::{Map, Value};
use url::Url;

const PUBLIC_MARKER: &str = "[REDACTED]";
const UPSTREAM_MARKER: &str = "__OPENCLAW_REDACTED__";
const INVALID_REDACTION: &str = "Invalid redacted settings value";

const NON_SENSITIVE_SUFFIXES: &[&str] = &[
    "maxtokens",
    "maxoutputtokens",
    "maxinputtokens",
    "maxcompletiontokens",
    "contexttokens",
    "totaltokens",
    "tokencount",
    "tokenlimit",
    "tokenbudget",
    "passwordfile",
];

/// Pure public projection. Trusted Settings consumers retain the original value.
pub(super) fn redact_public_settings(settings: &Value) -> Value {
    redact_value(settings, &ProjectionPath::default())
}

/// The Settings writer supplies its current document under its existing mutex.
/// This restores only submitted values; the owner still decides merge/removal.
pub(super) fn restore_public_settings_update(
    incoming: &Value,
    original: &Value,
) -> Result<Value, &'static str> {
    let restored = restore_value(incoming, Some(original), &ProjectionPath::default())?;
    if contains_marker(&restored) {
        return Err(INVALID_REDACTION);
    }
    Ok(restored)
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum FieldScope {
    #[default]
    Ordinary,
    Skills,
    SkillEntries,
    SkillEntry,
    LocalService,
    Environment,
}

impl FieldScope {
    fn child(self, key: &str) -> Self {
        if self == Self::Environment {
            Self::Environment
        } else if key.eq_ignore_ascii_case("env")
            && matches!(self, Self::SkillEntry | Self::LocalService)
        {
            Self::Environment
        } else if self == Self::SkillEntries {
            Self::SkillEntry
        } else if self == Self::Skills && key.eq_ignore_ascii_case("entries") {
            Self::SkillEntries
        } else if key.eq_ignore_ascii_case("skills") {
            Self::Skills
        } else if key.eq_ignore_ascii_case("localService") {
            Self::LocalService
        } else {
            Self::Ordinary
        }
    }
}

#[derive(Default)]
struct ProjectionPath {
    label: String,
    scope: FieldScope,
    sensitive: bool,
}

impl ProjectionPath {
    fn child(&self, key: &str) -> Self {
        let label = if self.label.is_empty() {
            key.to_string()
        } else {
            format!("{}.{key}", self.label)
        };
        let scope = self.scope.child(key);
        let lower = label.trim().to_lowercase();
        let environment = scope == FieldScope::Environment || lower.contains("localservice.env.");
        let whitelisted = NON_SENSITIVE_SUFFIXES
            .iter()
            .any(|suffix| lower.ends_with(suffix));
        let sensitive = environment
            || (!whitelisted && (self.sensitive || sensitive_name_pattern().is_match(&label)));
        Self { label, scope, sensitive }
    }

    fn array_item(&self) -> Self {
        Self {
            label: format!("{}[]", self.label),
            scope: match self.scope {
                FieldScope::SkillEntries => FieldScope::SkillEntry,
                FieldScope::Environment => FieldScope::Environment,
                _ => FieldScope::Ordinary,
            },
            sensitive: self.sensitive,
        }
    }
}

fn sensitive_name_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(
            r"(?i)(?:token$|password|secret|api.?key|encrypt.?key|encryption[_-]?key|private.?key|serviceaccount(?:ref)?$)",
        )
        .expect("constant settings sensitivity pattern")
    })
}

pub(super) fn is_marker(value: &Value) -> bool {
    value.as_str().is_some_and(|value| {
        matches!(value.trim(), PUBLIC_MARKER | UPSTREAM_MARKER)
    })
}

fn contains_marker(value: &Value) -> bool {
    match value {
        Value::Array(items) => items.iter().any(contains_marker),
        Value::Object(object) => object.values().any(contains_marker),
        _ => is_marker(value),
    }
}

fn is_secret_ref(object: &Map<String, Value>) -> bool {
    object.get("source").is_some_and(Value::is_string)
        && object.get("id").is_some_and(Value::is_string)
}

fn is_visible_ref_identity(key: &str, value: &Value) -> bool {
    matches!(key, "source" | "provider")
        && value.as_str().is_some_and(|value| !has_url_credentials(value))
}

fn should_mask_scalar(value: &Value, path: &ProjectionPath) -> bool {
    !value.is_object()
        && !value.is_array()
        && (path.sensitive || value.as_str().is_some_and(has_url_credentials))
}

fn redact_value(value: &Value, path: &ProjectionPath) -> Value {
    match value {
        Value::Object(object) => {
            let secret_ref = path.sensitive && is_secret_ref(object);
            Value::Object(object.iter().map(|(key, value)| {
                let projected = if secret_ref && is_visible_ref_identity(key, value) {
                    value.clone()
                } else {
                    redact_value(value, &path.child(key))
                };
                (key.clone(), projected)
            }).collect())
        }
        Value::Array(items) => {
            let item_path = path.array_item();
            Value::Array(items.iter().map(|item| redact_value(item, &item_path)).collect())
        }
        _ if should_mask_scalar(value, path) => Value::String(PUBLIC_MARKER.into()),
        _ => value.clone(),
    }
}

fn restore_value(
    incoming: &Value,
    original: Option<&Value>,
    path: &ProjectionPath,
) -> Result<Value, &'static str> {
    if is_marker(incoming) {
        let original = original.ok_or(INVALID_REDACTION)?;
        if !should_mask_scalar(original, path) {
            return Err(INVALID_REDACTION);
        }
        return Ok(original.clone());
    }
    match incoming {
        Value::Array(items) => restore_array(items, original, path),
        Value::Object(object) => {
            let original = original.and_then(Value::as_object);
            let secret_ref = path.sensitive
                && (is_secret_ref(object) || original.is_some_and(is_secret_ref));
            if secret_ref && object.get("id").is_some_and(is_marker) {
                require_unchanged_secret_ref_identity(object, original)?;
            }
            let mut restored = Map::new();
            for (key, value) in object {
                let restored_value = if secret_ref && is_visible_ref_identity(key, value) {
                    // These fields were visible, so a marker here is never restorable.
                    // The final marker check also refuses a forged masked identity.
                    value.clone()
                } else {
                    restore_value(value, original.and_then(|old| old.get(key)), &path.child(key))?
                };
                restored.insert(key.clone(), restored_value);
            }
            Ok(Value::Object(restored))
        }
        _ => Ok(incoming.clone()),
    }
}

fn require_unchanged_secret_ref_identity(
    incoming: &Map<String, Value>,
    original: Option<&Map<String, Value>>,
) -> Result<(), &'static str> {
    let original = original.filter(|value| is_secret_ref(value)).ok_or(INVALID_REDACTION)?;
    for key in ["source", "provider"] {
        let before = original.get(key).and_then(Value::as_str).ok_or(INVALID_REDACTION)?;
        let after = incoming.get(key).and_then(Value::as_str).ok_or(INVALID_REDACTION)?;
        if before != after || has_url_credentials(before) || has_url_credentials(after) {
            return Err(INVALID_REDACTION);
        }
    }
    Ok(())
}

struct ArrayIdentity {
    index: usize,
    count: usize,
}

fn literal_array_id(item: &Value) -> Option<&str> {
    let value = item.as_object()?.get("id")?;
    let id = value.as_str()?;
    if id.is_empty() || is_marker(value) || id.contains("$${") || has_env_reference(id) {
        return None;
    }
    Some(id)
}

fn has_env_reference(value: &str) -> bool {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        // Fixed-U substitutions use uppercase names and optional nonnested defaults.
        // An inner valid reference is still recognized inside an invalid outer one.
        Regex::new(r"\$\{[A-Z_][A-Z0-9_]*(?::-[^${}]*)?\}")
            .expect("constant settings environment reference pattern")
    }).is_match(value)
}

fn index_array_ids(items: &[Value]) -> BTreeMap<&str, ArrayIdentity> {
    let mut by_id = BTreeMap::<&str, ArrayIdentity>::new();
    for (index, item) in items.iter().enumerate() {
        if let Some(id) = literal_array_id(item) {
            by_id.entry(id).and_modify(|identity| identity.count += 1)
                .or_insert(ArrayIdentity { index, count: 1 });
        }
    }
    by_id
}

fn restore_array(
    incoming: &[Value],
    original: Option<&Value>,
    path: &ProjectionPath,
) -> Result<Value, &'static str> {
    let original = original.and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    let original_by_id = index_array_ids(original);
    let incoming_by_id = index_array_ids(incoming);
    let reserved = incoming_by_id.iter().filter_map(|(id, incoming_identity)| {
        let original_identity = original_by_id.get(id)?;
        (incoming_identity.count == 1 && original_identity.count == 1)
            .then_some(original_identity.index)
    }).collect::<BTreeSet<_>>();
    let has_unique_original = original_by_id.values().any(|identity| identity.count == 1);
    let item_path = path.array_item();
    let mut restored = Vec::with_capacity(incoming.len());
    for (index, item) in incoming.iter().enumerate() {
        let id = literal_array_id(item);
        let before = id.and_then(|id| original_by_id.get(id));
        let after = id.and_then(|id| incoming_by_id.get(id));
        let original_item = match (after, before) {
            (Some(after), Some(before)) if after.count == 1 && before.count == 1 => {
                original.get(before.index)
            }
            (Some(after), None) if after.count == 1 && has_unique_original => None,
            _ if !reserved.contains(&index) => original.get(index),
            _ => None,
        };
        restored.push(restore_value(item, original_item, &item_path)?);
    }
    Ok(Value::Array(restored))
}

fn has_url_credentials(value: &str) -> bool {
    let Ok(url) = Url::parse(value) else {
        return false;
    };
    !url.username().is_empty()
        || url.password().is_some_and(|password| !password.is_empty())
        || url.query_pairs().any(|(key, _)| is_sensitive_url_query_key(&key))
}

fn is_sensitive_url_query_key(key: &str) -> bool {
    static SEPARATORS: OnceLock<Regex> = OnceLock::new();
    static SCOPED_TOKEN: OnceLock<Regex> = OnceLock::new();
    let separators = SEPARATORS.get_or_init(|| {
        Regex::new(r"[\p{C}\p{Z}\u{115F}\u{1160}\u{3164}\u{FFA0}+]")
            .expect("constant URL query separator pattern")
    });
    let mut name = separators.replace_all(key, "").into_owned();
    // URL query parsing has already decoded one form layer. Fixed-U additionally
    // bounds percent decoding to nine passes and treats unresolved encoding as sensitive.
    for _ in 0..=8 {
        let Ok(decoded) = percent_decode_str(&name).decode_utf8() else {
            return true;
        };
        let decoded = separators.replace_all(&decoded, "");
        if decoded.as_ref() == name.as_str() {
            break;
        }
        name = decoded.into_owned();
    }
    if name.contains('%') {
        return true;
    }
    let name = name.trim().to_lowercase().replace('-', "_");
    matches!(name.as_str(),
        "token" | "key" | "api_key" | "apikey" | "secret" | "access_token"
        | "auth_token" | "password" | "pass" | "passwd" | "auth" | "jwt"
        | "session" | "id_token" | "code" | "client_secret" | "app_secret"
        | "hook_token" | "refresh_token" | "signature" | "x_amz_signature"
        | "x_amz_security_token" | "private_key" | "credential" | "authorization"
        | "sig" | "x_api_key" | "x_access_token" | "x_auth_token"
    ) || SCOPED_TOKEN.get_or_init(|| {
        Regex::new(r"(?:^|_)token(?:_[a-f0-9]{16,})?$")
            .expect("constant scoped URL token pattern")
    }).is_match(&name)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn restore(incoming: &Value, original: &Value) -> Value {
        match restore_public_settings_update(incoming, original) {
            Ok(value) => value,
            Err(_) => panic!("expected a valid masked settings update"),
        }
    }

    fn assert_rejected(incoming: &Value, original: &Value) {
        assert!(
            restore_public_settings_update(incoming, original).err() == Some(INVALID_REDACTION),
            "invalid masking must return only the bounded static error",
        );
    }

    #[test]
    fn named_scalars_are_masked_without_losing_ordinary_structure() {
        let original = json!({
            "agents": {"list": [{
                "id": "alpha", "model": {"provider": "example", "modelId": "model-a"},
                "authToken": "synthetic-a", "password": 17, "clientSecret": true,
                "api_key": "synthetic-b", "encryptKey": "synthetic-c",
                "encryptionKey": "synthetic-d", "encryption_key": "synthetic-e",
                "private-key": "synthetic-f", "serviceAccount": {"project": "synthetic-g"}
            }]},
            "userTimeZone": "Europe/London", "localToolPermission": "ask",
            "sidebarSections": [{"id": "main", "title": "Workspace", "isCollapsed": false}],
            "inferenceProvider": "example", "empty": null
        });
        let mut expected = original.clone();
        let entry = &mut expected["agents"]["list"][0];
        for field in ["authToken", "password", "clientSecret", "api_key", "encryptKey",
            "encryptionKey", "encryption_key", "private-key"] {
            entry[field] = json!(PUBLIC_MARKER);
        }
        entry["serviceAccount"]["project"] = json!(PUBLIC_MARKER);
        let projected = redact_public_settings(&original);
        assert!(projected == expected, "public projection must preserve ordinary structure");
        assert!(restore(&projected, &original) == original, "masked scalars must round-trip");
        assert!(original["agents"]["list"][0]["authToken"] != PUBLIC_MARKER,
            "projection must not mutate its trusted input");
    }

    #[test]
    fn fixed_u_whitelist_survives_sensitive_parents_but_not_environment_maps() {
        let exceptions = json!({
            "maxTokens": 12, "maxOutputTokens": 13, "maxInputTokens": 14,
            "maxCompletionTokens": 15, "contextTokens": 16, "totalTokens": 17,
            "tokenCount": 18, "tokenLimit": 19, "tokenBudget": 20,
            "passwordFile": "/configured/credential-file"
        });
        let original = json!({
            "secretOptions": exceptions,
            "ordinary": {"env": {"REGION": "west"}},
            "skills": {"entries": {"with.dots": {"enabled": true, "env": exceptions}}},
            "plugin": {"localService": {"env": {"REGION": "west", "maxTokens": 12}}}
        });
        let projected = redact_public_settings(&original);
        assert!(projected["secretOptions"] == exceptions,
            "fixed-U suffix exceptions must remain visible outside env");
        assert!(projected["ordinary"] == original["ordinary"],
            "ordinary environment-like metadata must remain unchanged");
        assert!(projected["skills"]["entries"]["with.dots"]["enabled"] == true,
            "skill metadata must stay visible");
        assert!(projected["skills"]["entries"]["with.dots"]["env"]
            .as_object().is_some_and(|env| env.values().all(|value| value == PUBLIC_MARKER)),
            "every skill env value is sensitive even when its name is whitelisted");
        assert!(projected["plugin"]["localService"]["env"]["REGION"] == PUBLIC_MARKER
            && projected["plugin"]["localService"]["env"]["maxTokens"] == PUBLIC_MARKER,
            "every local service env value is sensitive");
        assert!(restore(&projected, &original) == original, "env projection must round-trip");
    }

    #[test]
    fn inherited_sensitivity_masks_nested_arrays_and_all_scalar_types() {
        let original = json!({"apiKey": {
            "nested": [["synthetic", 7, true, null, ""]], "maxTokens": 32
        }});
        let projected = redact_public_settings(&original);
        assert!(projected == json!({"apiKey": {
            "nested": [[PUBLIC_MARKER, PUBLIC_MARKER, PUBLIC_MARKER, PUBLIC_MARKER, PUBLIC_MARKER]],
            "maxTokens": 32
        }}), "sensitive containers must keep structure while masking scalar leaves");
        assert!(restore(&projected, &original) == original, "scalar types must be restored exactly");
    }

    #[test]
    fn credentials_in_any_parsed_url_are_masked_as_a_whole() {
        let unsafe_urls = [
            "https://synthetic-user:synthetic-pass@example.test/path",
            "https://synthetic-user@example.test/path",
            "https://example.test/path?token=synthetic",
            "https://example.test/path?%61pi%5Fkey=synthetic",
            "https://example.test/path?%2561pi%255Fkey=synthetic",
            "https://example.test/path?x-api-key=synthetic",
            "https://example.test/path?sig=synthetic",
            "https://example.test/path?proxy_token_0123456789abcdef=synthetic",
            "https://example.test/path?to%E2%80%8Bken=synthetic",
            "https://example.test/path?to%E3%85%A4ken=synthetic",
            "https://example.test/path?to+ken=synthetic",
            "https://example.test/path?unresolved%25name=synthetic",
        ];
        for url in unsafe_urls {
            let original = json!({"ordinaryLabel": url, "items": [url]});
            let projected = redact_public_settings(&original);
            assert!(projected == json!({"ordinaryLabel": PUBLIC_MARKER, "items": [PUBLIC_MARKER]}),
                "credential URL must be masked without retaining a prefix or suffix");
            assert!(restore(&projected, &original) == original, "whole URL must round-trip");
        }
        let ordinary = json!({"endpoint": "https://example.test/path?page=2&maxTokens=32",
            "label": "provider:model", "url": "a plain non-URL description"});
        assert!(redact_public_settings(&ordinary) == ordinary, "safe ordinary strings stay visible");
        assert_rejected(&json!({"endpoint": PUBLIC_MARKER}), &ordinary);
    }

    #[test]
    fn both_reserved_markers_restore_only_existing_sensitive_scalar_locations() {
        let original = json!({"apiKey": "synthetic-a", "nested": {"password": "synthetic-b"}});
        for marker in [PUBLIC_MARKER, UPSTREAM_MARKER, "  [REDACTED]  "] {
            let incoming = json!({"apiKey": marker, "nested": {"password": marker}});
            assert!(restore(&incoming, &original) == original, "supported display markers must restore");
        }
        let explicit = json!({"apiKey": "replacement", "nested": {"password": ""}});
        assert!(restore(&explicit, &original) == explicit,
            "explicit replacements and empty removal requests must be preserved");
        let omitted = json!({"userTimeZone": "UTC"});
        assert!(restore(&omitted, &original) == omitted,
            "restoration must leave omission and merge semantics to Settings");
    }

    #[test]
    fn orphan_wrong_path_container_and_residual_markers_fail_closed() {
        let original = json!({"apiKey": "synthetic", "label": "visible", "passwords": {"one": "value"}});
        for incoming in [
            json!({"newSecret": PUBLIC_MARKER}),
            json!({"label": PUBLIC_MARKER}),
            json!({"nested": [UPSTREAM_MARKER]}),
            json!({"passwords": PUBLIC_MARKER}),
            json!(PUBLIC_MARKER),
        ] {
            assert_rejected(&incoming, &original);
        }
        for marker in [PUBLIC_MARKER, UPSTREAM_MARKER] {
            assert_rejected(&json!({"apiKey": marker}), &json!({"apiKey": marker}));
        }
        assert_rejected(&json!({"apiKey": [PUBLIC_MARKER]}), &json!({"apiKey": []}));
        assert!(original["apiKey"] == "synthetic", "refused restoration must not mutate the source");
    }

    #[test]
    fn sensitive_secret_refs_keep_visible_identity_and_restore_the_id() {
        let original = json!({"apiKey": {
            "source": "env", "provider": "deployment", "id": "SYNTHETIC_ENV_NAME",
            "maxTokens": 64
        }});
        let projected = redact_public_settings(&original);
        assert!(projected == json!({"apiKey": {
            "source": "env", "provider": "deployment", "id": PUBLIC_MARKER, "maxTokens": 64
        }}), "SecretRef identity must remain visible while its id is masked");
        assert!(restore(&projected, &original) == original, "unchanged SecretRef must round-trip");
        let nonsensitive = json!({"descriptor": {"source": "catalog", "provider": "example", "id": "visible"}});
        assert!(redact_public_settings(&nonsensitive) == nonsensitive,
            "ordinary source/provider/id records must stay visible");
        assert_rejected(&json!({"descriptor": {"source": "catalog", "provider": "example", "id": PUBLIC_MARKER}}),
            &nonsensitive);
    }

    #[test]
    fn masked_secret_ref_ids_require_unchanged_explicit_source_and_provider() {
        let original = json!({"apiKey": {"source": "env", "provider": "deployment", "id": "SYNTHETIC_ENV_NAME"}});
        for reference in [
            json!({"source": "file", "provider": "deployment", "id": PUBLIC_MARKER}),
            json!({"source": "env", "provider": "other", "id": PUBLIC_MARKER}),
            json!({"source": "env", "id": PUBLIC_MARKER}),
            json!({"provider": "deployment", "id": PUBLIC_MARKER}),
            json!({"source": PUBLIC_MARKER, "provider": PUBLIC_MARKER, "id": PUBLIC_MARKER}),
            json!({"source": "env", "provider": "deployment", "id": PUBLIC_MARKER, "extra": UPSTREAM_MARKER}),
        ] {
            assert_rejected(&json!({"apiKey": reference}), &original);
        }
        assert_rejected(&json!({"apiKey": {"source": "env", "provider": "deployment", "id": PUBLIC_MARKER}}),
            &json!({"apiKey": {"source": "env", "id": "SYNTHETIC_ENV_NAME"}}));
        let explicit = json!({"apiKey": {"source": "file", "provider": "other", "id": "explicit-new-reference"}});
        assert!(restore(&explicit, &original) == explicit,
            "changing a SecretRef backend requires an explicit replacement id");
    }

    #[test]
    fn a_visible_secret_ref_identity_cannot_be_restored_by_omitting_its_id() {
        let original = json!({"apiKey": {"source": "env", "provider": "deployment", "id": "SYNTHETIC_ENV_NAME"}});
        assert_rejected(&json!({"apiKey": {"source": PUBLIC_MARKER}}), &original);
        assert_rejected(&json!({"apiKey": {"provider": UPSTREAM_MARKER}}), &original);
    }

    #[test]
    fn credential_urls_in_secret_ref_identity_are_masked_and_cannot_auto_restore() {
        for field in ["source", "provider"] {
            let mut original = json!({"apiKey": {
                "source": "env", "provider": "deployment", "id": "SYNTHETIC_ENV_NAME"
            }});
            original["apiKey"][field] = json!("https://synthetic-user:synthetic-pass@example.test/");
            let projected = redact_public_settings(&original);
            assert!(projected["apiKey"][field] == PUBLIC_MARKER,
                "malformed SecretRef identity cannot bypass URL protection");
            assert_rejected(&projected, &original);
            let mut explicit_identity = original.clone();
            explicit_identity["apiKey"]["id"] = json!(PUBLIC_MARKER);
            assert_rejected(&explicit_identity, &original);
            let repaired = json!({"apiKey": {
                "source": "env", "provider": "deployment", "id": "EXPLICIT_REPAIRED_ID"
            }});
            assert!(restore(&repaired, &original) == repaired,
                "an explicit repaired identity and id remain accepted");
        }
    }

    #[test]
    fn reordered_unique_array_ids_restore_their_own_secrets() {
        let original = json!({"agents": {"list": [
            {"id": "alpha", "apiKey": "synthetic-a", "name": "A"},
            {"id": "beta", "apiKey": "synthetic-b", "name": "B"}
        ]}});
        let incoming = json!({"agents": {"list": [
            {"id": "beta", "apiKey": PUBLIC_MARKER, "name": "changed B"},
            {"id": "alpha", "apiKey": UPSTREAM_MARKER, "name": "changed A"}
        ]}});
        let expected = json!({"agents": {"list": [
            {"id": "beta", "apiKey": "synthetic-b", "name": "changed B"},
            {"id": "alpha", "apiKey": "synthetic-a", "name": "changed A"}
        ]}});
        assert!(restore(&incoming, &original) == expected,
            "reordering must bind secrets to literal unique ids instead of old indexes");
    }

    #[test]
    fn a_new_unique_array_id_cannot_borrow_an_identified_original_secret() {
        let original = json!({"items": [{"id": "alpha", "apiKey": "synthetic-a"}]});
        assert_rejected(&json!({"items": [{"id": "new", "apiKey": PUBLIC_MARKER}]}), &original);
        let explicit = json!({"items": [{"id": "new", "apiKey": "explicit-new-value"}]});
        assert!(restore(&explicit, &original) == explicit, "new entries may supply explicit credentials");
        let mixed = json!({"items": [{"apiKey": "synthetic-unidentified"}, {"id": "alpha", "apiKey": "synthetic-a"}]});
        assert_rejected(&json!({"items": [{"id": "new", "apiKey": PUBLIC_MARKER}]}), &mixed);
    }

    #[test]
    fn literal_array_id_matching_does_not_trim_or_fold_case() {
        let original = json!({"items": [
            {"id": "alpha", "apiKey": "synthetic-first"},
            {"id": "ALPHA", "apiKey": "synthetic-second"},
            {"id": " alpha ", "apiKey": "synthetic-third"}
        ]});
        let incoming = json!({"items": [
            {"id": " alpha ", "apiKey": PUBLIC_MARKER},
            {"id": "alpha", "apiKey": PUBLIC_MARKER},
            {"id": "ALPHA", "apiKey": PUBLIC_MARKER}
        ]});
        assert!(restore(&incoming, &original) == json!({"items": [
            original["items"][2].clone(), original["items"][0].clone(), original["items"][1].clone()
        ]}), "array identities must stay literal rather than applying roster normalization");
    }

    #[test]
    fn matched_old_indexes_are_reserved_before_any_positional_fallback() {
        let original = json!({"items": [
            {"id": "alpha", "apiKey": "synthetic-a"},
            {"id": "beta", "apiKey": "synthetic-b"}
        ]});
        assert_rejected(&json!({"items": [
            {"apiKey": PUBLIC_MARKER}, {"id": "alpha", "apiKey": PUBLIC_MARKER}
        ]}), &original);
        assert_rejected(&json!({"items": [
            {"apiKey": PUBLIC_MARKER}, {"id": "alpha", "apiKey": "explicit-replacement"}
        ]}), &original);
        assert_rejected(&json!({"items": [
            {"id": "beta", "apiKey": PUBLIC_MARKER}, {"apiKey": PUBLIC_MARKER}
        ]}), &original);
        let allowed = json!({"items": [
            {"id": "alpha", "apiKey": PUBLIC_MARKER}, {"apiKey": PUBLIC_MARKER}
        ]});
        assert!(restore(&allowed, &original)["items"][1]["apiKey"] == original["items"][1]["apiKey"],
            "unreserved positions retain fixed-U no-id fallback");
    }

    #[test]
    fn duplicate_ids_use_bounded_positional_fallback_instead_of_a_unique_match() {
        let original = json!({"items": [
            {"id": "duplicate", "apiKey": "synthetic-first"},
            {"id": "duplicate", "apiKey": "synthetic-second"}
        ]});
        let incoming = json!({"items": [
            {"id": "duplicate", "apiKey": PUBLIC_MARKER},
            {"id": "duplicate", "apiKey": PUBLIC_MARKER}
        ]});
        assert!(restore(&incoming, &original) == original,
            "duplicate identities must restore each unreserved old position independently");
        let changed = json!({"items": [
            {"id": "new", "apiKey": "explicit"},
            {"id": "duplicate", "apiKey": PUBLIC_MARKER}
        ]});
        assert!(restore(&changed, &original)["items"][1]["apiKey"] == original["items"][1]["apiKey"],
            "one remaining incoming duplicate does not make the old duplicate identity unique");
        let new_literal = json!({"items": [{"id": "new", "apiKey": PUBLIC_MARKER}]});
        assert!(restore(&new_literal, &original)["items"][0]["apiKey"] == original["items"][0]["apiKey"],
            "fixed-U fallback remains positional when the old array contains only duplicate ids");
        let incoming_duplicates = json!({"items": [
            {"id": "alpha", "apiKey": PUBLIC_MARKER},
            {"id": "alpha", "apiKey": PUBLIC_MARKER}
        ]});
        let distinct_originals = json!({"items": [
            {"id": "alpha", "apiKey": "synthetic-first"},
            {"id": "beta", "apiKey": "synthetic-second"}
        ]});
        let restored = restore(&incoming_duplicates, &distinct_originals);
        assert!(restored["items"][0]["apiKey"] == distinct_originals["items"][0]["apiKey"]
            && restored["items"][1]["apiKey"] == distinct_originals["items"][1]["apiKey"],
            "duplicate incoming ids also retain positional fallback rather than unique matching");
    }

    #[test]
    fn no_id_fallback_is_positional_and_cannot_restore_an_appended_orphan() {
        let original = json!({"items": [{"apiKey": "synthetic-first"}, {"apiKey": "synthetic-second"}]});
        let incoming = json!({"items": [{"apiKey": PUBLIC_MARKER}, {"apiKey": PUBLIC_MARKER}]});
        assert!(restore(&incoming, &original) == original, "unidentified items retain positional fallback");
        let truncated = json!({"items": [{"apiKey": PUBLIC_MARKER}]});
        assert!(restore(&truncated, &original) == json!({"items": [original["items"][0].clone()]}),
            "array truncation must not invent omitted entries");
        assert_rejected(&json!({"items": [
            {"apiKey": PUBLIC_MARKER}, {"apiKey": PUBLIC_MARKER}, {"apiKey": PUBLIC_MARKER}
        ]}), &original);
        let new_literal = json!({"items": [{"id": "new", "apiKey": PUBLIC_MARKER}]});
        assert!(restore(&new_literal, &original)["items"][0]["apiKey"] == original["items"][0]["apiKey"],
            "a legacy array without any unique old identity retains fixed-U fallback");
    }

    #[test]
    fn environment_and_escaped_ids_never_become_unique_array_identities() {
        for (first, second) in [
            ("${FIRST}", "${SECOND}"),
            ("${FIRST:-default}", "${SECOND:-default}"),
            ("$${FIRST}", "$${SECOND}"),
            ("prefix-${FIRST}", "prefix-${SECOND}"),
            ("${OUTER:-${FIRST}}", "${OUTER:-${SECOND}}"),
        ] {
            let original = json!({"items": [
                {"id": first, "apiKey": "synthetic-first"},
                {"id": second, "apiKey": "synthetic-second"}
            ]});
            let incoming = json!({"items": [
                {"id": second, "apiKey": PUBLIC_MARKER},
                {"id": first, "apiKey": PUBLIC_MARKER}
            ]});
            let restored = restore(&incoming, &original);
            assert!(restored["items"][0]["apiKey"] == original["items"][0]["apiKey"]
                && restored["items"][1]["apiKey"] == original["items"][1]["apiKey"],
                "authored environment expressions must use bounded fallback, never id matching");
        }
    }

    #[test]
    fn empty_nonstring_and_reserved_ids_are_not_stable_array_identities() {
        for id in [json!(""), Value::Null, json!(7)] {
            let original = json!({"items": [{"id": id, "apiKey": "synthetic-first"}]});
            let incoming = json!({"items": [{"id": id, "apiKey": PUBLIC_MARKER}]});
            assert!(restore(&incoming, &original) == original, "nonliteral ids must preserve positional fallback");
        }
        for marker in [PUBLIC_MARKER, UPSTREAM_MARKER] {
            assert_rejected(&json!({"items": [{"id": marker, "apiKey": PUBLIC_MARKER}]}),
                &json!({"items": [{"id": "literal", "apiKey": "synthetic-first"}]}));
        }
    }

    #[test]
    fn array_reordering_preserves_secret_ref_identity_guards() {
        let original = json!({"items": [
            {"id": "alpha", "apiKey": {"source": "env", "provider": "one", "id": "SYNTHETIC_A"}},
            {"id": "beta", "apiKey": {"source": "file", "provider": "two", "id": "SYNTHETIC_B"}}
        ]});
        let projected = redact_public_settings(&original);
        let incoming = json!({"items": [projected["items"][1].clone(), projected["items"][0].clone()]});
        let expected = json!({"items": [original["items"][1].clone(), original["items"][0].clone()]});
        assert!(restore(&incoming, &original) == expected, "array identity and SecretRef identity must compose");
        let mut changed = incoming;
        changed["items"][0]["apiKey"]["provider"] = json!("changed");
        assert_rejected(&changed, &original);
    }
}
