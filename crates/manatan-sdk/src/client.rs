use serde::de::DeserializeOwned;

use crate::{net, Error, Result};

#[derive(Clone, Debug, Default)]
pub struct Client {
    headers: Vec<(String, String)>,
    cookie_url: Option<String>,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn browser() -> Self {
        Self::new()
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/138 Safari/537.36",
            )
            .header("Accept", "text/html,application/xhtml+xml,application/json;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn cookies_for(mut self, url: impl Into<String>) -> Self {
        self.cookie_url = Some(url.into());
        self
    }

    pub fn get(&self, url: impl Into<String>) -> RequestBuilder {
        self.request("GET", url)
    }

    pub fn post(&self, url: impl Into<String>) -> RequestBuilder {
        self.request("POST", url)
    }

    pub fn request(&self, method: impl Into<String>, url: impl Into<String>) -> RequestBuilder {
        RequestBuilder {
            request: net::Request {
                method: method.into(),
                url: url.into(),
                cookie_url: self.cookie_url.clone(),
                headers: self.headers.clone(),
                body: None,
                timeout_ms: None,
                redirect_policy: None,
                max_body_bytes: None,
                rate_limit_key: None,
                minimum_interval_ms: None,
            },
        }
    }

    /// Execute independent requests concurrently in the host. This is the
    /// portable replacement for coroutine fan-out in extractors.
    pub fn send_many(requests: Vec<RequestBuilder>, concurrency: u16) -> Vec<Result<Response>> {
        let requests = requests
            .into_iter()
            .map(|request| request.request)
            .collect::<Vec<_>>();
        net::fetch_many(&requests, concurrency.clamp(1, 16))
            .into_iter()
            .map(|result| result.map(Response).map_err(Error::new))
            .collect()
    }
}

pub struct RequestBuilder {
    request: net::Request,
}

impl RequestBuilder {
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.request.headers.push((name.into(), value.into()));
        self
    }

    pub fn cookies_for(mut self, url: impl Into<String>) -> Self {
        self.request.cookie_url = Some(url.into());
        self
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.request.body = Some(body.into());
        self
    }

    pub fn timeout_ms(mut self, timeout_ms: u32) -> Self {
        self.request.timeout_ms = Some(timeout_ms);
        self
    }

    /// Set redirect handling to `follow` (default), `manual`, or `error`.
    pub fn redirect_policy(mut self, policy: impl Into<String>) -> Self {
        self.request.redirect_policy = Some(policy.into());
        self
    }

    pub fn max_body_bytes(mut self, max_body_bytes: u64) -> Self {
        self.request.max_body_bytes = Some(max_body_bytes);
        self
    }

    /// Coordinate this request with other calls from the same source. The
    /// host persists the next allowed instant by key across component
    /// instances, unlike guest-local sleeps.
    pub fn rate_limit(mut self, key: impl Into<String>, minimum_interval_ms: u32) -> Self {
        self.request.rate_limit_key = Some(key.into());
        self.request.minimum_interval_ms = Some(minimum_interval_ms);
        self
    }

    pub fn json<T: serde::Serialize>(self, value: &T) -> Result<Self> {
        let body = serde_json::to_vec(value)?;
        Ok(self.header("Content-Type", "application/json").body(body))
    }

    pub fn form(self, values: &[(&str, &str)]) -> Self {
        let body = values
            .iter()
            .map(|(key, value)| {
                format!(
                    "{}={}",
                    url::form_urlencoded::byte_serialize(key.as_bytes()).collect::<String>(),
                    url::form_urlencoded::byte_serialize(value.as_bytes()).collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("&");
        self.header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.into_bytes())
    }

    pub fn send(self) -> Result<Response> {
        net::fetch(&self.request).map(Response).map_err(Error::new)
    }
}

pub struct Response(pub net::Response);

impl Response {
    pub fn status(&self) -> u16 {
        self.0.status
    }

    pub fn final_url(&self) -> &str {
        &self.0.final_url
    }

    pub fn headers(&self) -> &[(String, String)] {
        &self.0.headers
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0.body
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0.body
    }

    pub fn text(&self) -> Result<&str> {
        std::str::from_utf8(&self.0.body).map_err(|error| Error::new(error.to_string()))
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.0.body).map_err(Error::from)
    }

    pub fn error_for_status(self) -> Result<Self> {
        if (200..400).contains(&self.0.status) {
            Ok(self)
        } else {
            Err(Error::new(format!(
                "HTTP {} from {}",
                self.0.status, self.0.final_url
            )))
        }
    }
}
