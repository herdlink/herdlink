use std::path::Path;

use serde::Deserialize;

use crate::{Error, Result};

// Deliberately no Debug/Serialize: these types contain credentials.
pub(crate) enum Auth {
    ApiKey(String),
    Codex {
        access_token: String,
        account_id: String,
    },
}

#[derive(Deserialize)]
struct AuthFile {
    auth_mode: Option<String>,
    #[serde(rename = "OPENAI_API_KEY", alias = "api_key")]
    api_key: Option<String>,
    tokens: Option<Tokens>,
}

#[derive(Deserialize)]
struct Tokens {
    access_token: String,
    account_id: String,
}

impl Auth {
    pub(crate) fn from_file(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(Error::AuthFileIo)?;
        // Serde's detailed errors can contain input values; do not expose them.
        let file: AuthFile = serde_json::from_slice(&bytes)
            .map_err(|_| Error::Configuration("invalid auth.json format".into()))?;
        match file.auth_mode.as_deref() {
            Some("apikey") => Self::api_key(file.api_key),
            Some("chatgpt") => Self::codex(file.tokens),
            None => {
                if file
                    .api_key
                    .as_ref()
                    .is_some_and(|key| !key.trim().is_empty())
                {
                    Self::api_key(file.api_key)
                } else {
                    Self::codex(file.tokens)
                }
            }
            Some(_) => Err(Error::Configuration(
                "unsupported auth.json auth_mode".into(),
            )),
        }
    }

    fn api_key(key: Option<String>) -> Result<Self> {
        match key {
            Some(key) if !key.trim().is_empty() => Ok(Self::ApiKey(key)),
            _ => Err(Error::Configuration(
                "auth.json is missing an API key".into(),
            )),
        }
    }

    fn codex(tokens: Option<Tokens>) -> Result<Self> {
        match tokens {
            Some(tokens)
                if !tokens.access_token.trim().is_empty()
                    && !tokens.account_id.trim().is_empty() =>
            {
                Ok(Self::Codex {
                    access_token: tokens.access_token,
                    account_id: tokens.account_id,
                })
            }
            _ => Err(Error::Configuration(
                "auth.json needs an API key or tokens.access_token and tokens.account_id".into(),
            )),
        }
    }

    pub(crate) fn token(&self) -> &str {
        match self {
            Self::ApiKey(key) => key,
            Self::Codex { access_token, .. } => access_token,
        }
    }

    pub(crate) fn is_codex(&self) -> bool {
        matches!(self, Self::Codex { .. })
    }

    pub(crate) fn base_url(&self) -> &'static str {
        match self {
            Self::ApiKey(_) => "https://api.openai.com/v1",
            Self::Codex { .. } => "https://chatgpt.com/backend-api/codex",
        }
    }
}
