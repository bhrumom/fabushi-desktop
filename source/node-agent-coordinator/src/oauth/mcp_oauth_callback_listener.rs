use std::collections::HashMap;

use crate::protocol::Failure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
    pub server_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthCallbackDisposition {
    Unhandled,
    ProviderError {
        state: String,
        server_name: String,
        error: String,
    },
    MissingCode {
        state: String,
        server_name: String,
    },
    Complete(OAuthCallback),
}

impl OAuthCallback {
    pub fn validate(self) -> Result<Self, Failure> {
        if self.code.trim().is_empty()
            || self.state.trim().is_empty()
            || self.server_name.trim().is_empty()
        {
            return Err(Failure::new(
                "MCP_OAUTH_INVALID_CALLBACK",
                "OAuth callback requires code, state, and server identity",
            ));
        }
        Ok(self)
    }
}

pub fn classify_callback(
    request_target: &str,
    expected_path: &str,
    mut resolve: impl FnMut(&str) -> Option<String>,
) -> Result<OAuthCallbackDisposition, Failure> {
    let (path, query) = request_target
        .split_once('?')
        .map_or((request_target, ""), |(path, query)| (path, query));
    if path != expected_path {
        return Ok(OAuthCallbackDisposition::Unhandled);
    }

    let params = parse_query(query)?;
    let Some(state) = params.get("state").filter(|state| !state.is_empty()).cloned() else {
        return Ok(OAuthCallbackDisposition::Unhandled);
    };
    let Some(server_name) = resolve(&state) else {
        return Ok(OAuthCallbackDisposition::Unhandled);
    };

    if let Some(error) = params.get("error") {
        return Ok(OAuthCallbackDisposition::ProviderError {
            state,
            server_name,
            error: error.clone(),
        });
    }

    let Some(code) = params.get("code").filter(|code| !code.is_empty()).cloned() else {
        return Ok(OAuthCallbackDisposition::MissingCode {
            state,
            server_name,
        });
    };

    Ok(OAuthCallbackDisposition::Complete(
        OAuthCallback {
            code,
            state,
            server_name,
        }
        .validate()?,
    ))
}

fn parse_query(query: &str) -> Result<HashMap<String, String>, Failure> {
    let mut params = HashMap::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (raw_key, raw_value) = pair
            .split_once('=')
            .map_or((pair, ""), |(key, value)| (key, value));
        let key = decode_component(raw_key)?;
        let value = decode_component(raw_value)?;
        params.entry(key).or_insert(value);
    }
    Ok(params)
}

fn decode_component(value: &str) -> Result<String, Failure> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(Failure::new(
                        "MCP_OAUTH_INVALID_CALLBACK",
                        "OAuth callback query contains malformed percent encoding",
                    ));
                }
                let high = hex(bytes[index + 1]).ok_or_else(|| {
                    Failure::new(
                        "MCP_OAUTH_INVALID_CALLBACK",
                        "OAuth callback query contains malformed percent encoding",
                    )
                })?;
                let low = hex(bytes[index + 2]).ok_or_else(|| {
                    Failure::new(
                        "MCP_OAUTH_INVALID_CALLBACK",
                        "OAuth callback query contains malformed percent encoding",
                    )
                })?;
                output.push((high << 4) | low);
                index += 3;
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(output).map_err(|_| {
        Failure::new(
            "MCP_OAUTH_INVALID_CALLBACK",
            "OAuth callback query is not valid UTF-8",
        )
    })
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
