use std::{collections::BTreeMap, path::Path, pin::Pin, time::Duration};

use eventsource_stream::Eventsource;
use futures_core::Stream;
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use crate::auth::Auth;
use crate::{Error, Event, Response, ResponseRequest, Result};

pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<Event>> + Send>>;

/// Clones share an HTTP connection pool. Credentials are never included in Debug.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    endpoint: reqwest::Url,
    codex_auth: bool,
}

pub struct ClientBuilder {
    auth: Auth,
    base_url: String,
    timeout: Duration,
}

impl ClientBuilder {
    /// Use an API root, e.g. `https://api.openai.com/v1`.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Total request deadline, including reading the streamed body.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<Client> {
        if self.auth.token().trim().is_empty() {
            return Err(Error::Configuration("authentication token is empty".into()));
        }
        let mut authorization = HeaderValue::from_str(&format!("Bearer {}", self.auth.token()))
            .map_err(|_| {
                Error::Configuration(
                    "authentication token contains invalid header characters".into(),
                )
            })?;
        authorization.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authorization);
        if let Auth::Codex { account_id, .. } = &self.auth {
            let mut account = HeaderValue::from_str(account_id)
                .map_err(|_| Error::Configuration("invalid auth.json account ID".into()))?;
            account.set_sensitive(true);
            headers.insert("chatgpt-account-id", account);
        }
        let mut endpoint = reqwest::Url::parse(&self.base_url)
            .map_err(|_| Error::Configuration("base URL is invalid".into()))?;
        if !matches!(endpoint.scheme(), "http" | "https")
            || endpoint.host_str().is_none()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
        {
            return Err(Error::Configuration(
                "base URL must be HTTP(S), without credentials, query, or fragment".into(),
            ));
        }
        endpoint.set_path(&format!(
            "{}/responses",
            endpoint.path().trim_end_matches('/')
        ));
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(Duration::from_secs(30))
            .timeout(self.timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Client {
            http,
            endpoint,
            codex_auth: self.auth.is_codex(),
        })
    }
}

impl Client {
    pub fn builder(api_key: impl Into<String>) -> ClientBuilder {
        Self::with_auth(Auth::ApiKey(api_key.into()))
    }

    fn with_auth(auth: Auth) -> ClientBuilder {
        ClientBuilder {
            base_url: auth.base_url().into(),
            auth,
            timeout: Duration::from_secs(300),
        }
    }

    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).build()
    }

    /// Reads a Codex auth.json containing an API key or ChatGPT login tokens.
    /// Credentials are loaded once; rebuild after Codex refreshes the file.
    pub fn from_auth_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::builder_from_auth_file(path)?.build()
    }

    pub fn builder_from_auth_file(path: impl AsRef<Path>) -> Result<ClientBuilder> {
        Ok(Self::with_auth(Auth::from_file(path.as_ref())?))
    }

    /// `OPENAI_AUTH_FILE` takes precedence over `OPENAI_API_KEY` when set.
    /// `OPENAI_BASE_URL` optionally overrides the authentication-specific API root.
    pub fn from_env() -> Result<Self> {
        let mut builder = if let Some(path) = std::env::var_os("OPENAI_AUTH_FILE") {
            Self::builder_from_auth_file(path)?
        } else {
            let key = std::env::var("OPENAI_API_KEY").map_err(|_| {
                Error::Configuration("set OPENAI_AUTH_FILE or OPENAI_API_KEY".into())
            })?;
            Self::builder(key)
        };
        match std::env::var("OPENAI_BASE_URL") {
            Ok(url) => builder = builder.base_url(url),
            Err(std::env::VarError::NotPresent) => {}
            Err(_) => return Err(Error::Configuration("OPENAI_BASE_URL is invalid".into())),
        }
        builder.build()
    }

    async fn send(&self, request: &ResponseRequest, stream: bool) -> Result<reqwest::Response> {
        let mut body = serde_json::to_value(request)?;
        body["stream"] = stream.into();
        if request.store == Some(false) {
            body["include"] = serde_json::json!(["reasoning.encrypted_content"]);
        }
        if self.codex_auth {
            if request.previous_response_id.is_some() || request.store == Some(true) {
                return Err(Error::Configuration(
                    "ChatGPT login requires store=false and stateless continuation".into(),
                ));
            }
            if request.max_output_tokens.is_some() {
                return Err(Error::Configuration(
                    "max_output_tokens is not supported by the Codex backend".into(),
                ));
            }
            body["store"] = false.into();
            body["instructions"] = request.instructions.as_deref().unwrap_or("").into();
            body["include"] = serde_json::json!(["reasoning.encrypted_content"]);
            if let crate::Input::Text(text) = &request.input {
                body["input"] =
                    serde_json::json!([{"type": "message", "role": "user", "content": text}]);
            }
        }
        let response = self
            .http
            .post(self.endpoint.clone())
            .header(
                "Accept",
                if stream {
                    "text/event-stream"
                } else {
                    "application/json"
                },
            )
            .json(&body)
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let request_id = response
                .headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned);
            return Err(Error::Http {
                status,
                request_id,
                body: response.text().await?,
            });
        }
        Ok(response)
    }

    pub async fn create(&self, request: &ResponseRequest) -> Result<Response> {
        // Codex's backend always streams; preserve the non-streaming public API
        // by consuming the stream and returning its completed response.
        if self.codex_auth {
            let mut events = self.stream(request).await?;
            while let Some(event) = events.next().await {
                if let Event::Completed { response } = event? {
                    return Ok(response);
                }
            }
            return Err(Error::UnexpectedEof);
        }
        let response: Response = self.send(request, false).await?.json().await?;
        if response.status != "completed" {
            return Err(Error::UnsuccessfulResponse(Box::new(response)));
        }
        Ok(response)
    }

    /// Opens the SSE stream without buffering the whole response. Dropping the
    /// stream closes the local connection; no server-side cancellation is sent.
    /// Failures and incomplete responses yield an error and end the stream.
    pub async fn stream(&self, request: &ResponseRequest) -> Result<ResponseStream> {
        let codex_auth = self.codex_auth;
        let response = self.send(request, true).await?;
        let content_type = response
            .headers()
            .get("content-type")
            .map(|value| value.to_str().unwrap_or("<invalid header>"));
        // The Codex endpoint can return valid SSE without Content-Type. In that
        // case validate the actual events and require a terminal response below.
        // An explicitly incompatible type still indicates a different payload.
        if let Some(content_type) = content_type
            && !content_type
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .eq_ignore_ascii_case("text/event-stream")
        {
            return Err(Error::Stream(format!(
                "expected text/event-stream content type, received {content_type:?} (HTTP {})",
                response.status().as_u16()
            )));
        }
        Ok(Box::pin(async_stream::try_stream! {
            let mut completed_items = BTreeMap::new();
            let events = response.bytes_stream().eventsource();
            futures_util::pin_mut!(events);
            while let Some(event) = events.next().await {
                let event = event.map_err(|error| Error::Stream(error.to_string()))?;
                if event.data == "[DONE]" {
                    break;
                }
                let payload: serde_json::Value = serde_json::from_str(&event.data)?;
                let raw_item = payload.get("item").cloned();
                let event: Event = serde_json::from_value(payload)?;
                match event {
                    Event::Failed { mut response } | Event::Incomplete { mut response } => {
                        response.restore_stream_output(completed_items.into_values().collect())?;
                        Err(Error::UnsuccessfulResponse(Box::new(response)))?;
                        return;
                    }
                    Event::Error { code, message } => {
                        Err(Error::Api { code, message })?;
                    }
                    Event::Completed { mut response } => {
                        response.restore_stream_output(completed_items.into_values().collect())?;
                        if codex_auth {
                            response.store = Some(false);
                        }
                        if response.status != "completed" {
                            Err(Error::UnsuccessfulResponse(Box::new(response)))?;
                        } else {
                            yield Event::Completed { response };
                            return;
                        }
                        return;
                    }
                    Event::OutputItemDone { output_index, item } => {
                        if let Some(raw_item) = raw_item {
                            completed_items.insert(output_index, raw_item);
                        }
                        yield Event::OutputItemDone { output_index, item };
                    }
                    Event::Other => {}
                    event => yield event,
                }
            }
            Err(Error::UnexpectedEof)?;
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authentication_selects_the_correct_default_endpoint() {
        let api = Client::new("test-key").unwrap();
        assert_eq!(api.endpoint.as_str(), "https://api.openai.com/v1/responses");
        for mode in [serde_json::json!("chatgpt"), serde_json::Value::Null] {
            let file = tempfile::NamedTempFile::new().unwrap();
            let auth = serde_json::json!({
                "auth_mode": mode,
                "OPENAI_API_KEY": null,
                "tokens": {"access_token": "test-token", "account_id": "test-account"}
            });
            std::fs::write(file.path(), auth.to_string()).unwrap();
            let client = Client::from_auth_file(file.path()).unwrap();
            assert_eq!(
                client.endpoint.as_str(),
                "https://chatgpt.com/backend-api/codex/responses"
            );
            assert!(client.codex_auth);
        }
    }
}
