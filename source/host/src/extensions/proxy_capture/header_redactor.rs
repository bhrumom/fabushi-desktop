use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ProxyHeaderRedactor {
    sensitive_headers: HashSet<String>,
}

impl Default for ProxyHeaderRedactor {
    fn default() -> Self {
        let mut set = HashSet::new();
        for h in &[
            "authorization",
            "proxy-authorization",
            "x-api-key",
            "cookie",
            "set-cookie",
            "proxy-authenticate",
            "x-auth-token",
        ] {
            set.insert(h.to_string());
        }
        Self {
            sensitive_headers: set,
        }
    }
}

impl ProxyHeaderRedactor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn redact_headers(&self, headers: &HashMap<String, String>) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for (k, v) in headers {
            if self.sensitive_headers.contains(&k.to_ascii_lowercase()) {
                result.insert(k.clone(), "[REDACTED]".to_string());
            } else {
                result.insert(k.clone(), v.clone());
            }
        }
        result
    }

    pub fn is_sensitive_header(&self, header_name: &str) -> bool {
        self.sensitive_headers
            .contains(&header_name.to_ascii_lowercase())
    }
}
