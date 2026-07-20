use serde::de::DeserializeOwned;

use crate::{browser, net, Error, Result};

pub const BROWSER_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/138 Safari/537.36";

/// A bounded, host-browser fallback for an HTTP anti-bot challenge.
///
/// The normal HTTP request always runs first. A WebView is opened only when
/// the response matches the selected provider's challenge signature, and the
/// original request is retried at most once with host-managed cookies. The
/// WebView profile and cookies remain isolated to the current extension
/// source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserChallengePolicy {
    kind: BrowserChallengeKind,
    cookie_url: String,
    profile: String,
    timeout_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BrowserChallengeKind {
    Cloudflare,
}

impl BrowserChallengePolicy {
    /// Recover Cloudflare challenge pages using an origin-scoped WebView.
    /// `cookie_url` must share an origin with every request using this policy.
    pub fn cloudflare(cookie_url: impl Into<String>) -> Self {
        Self {
            kind: BrowserChallengeKind::Cloudflare,
            cookie_url: cookie_url.into(),
            profile: "cloudflare".to_string(),
            timeout_ms: 30_000,
        }
    }

    /// Override the source-local persistent browser profile id.
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = profile.into();
        self
    }

    /// Override the browser challenge timeout. The host applies its own hard
    /// upper bound in addition to this value.
    pub fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn matches(&self, response: &Response) -> bool {
        match self.kind {
            BrowserChallengeKind::Cloudflare => is_cloudflare_challenge(response),
        }
    }

    fn webview_request(&self, request: &net::Request) -> browser::WebViewRequest {
        let user_agent = request
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("user-agent"))
            .map(|(_, value)| value.clone());
        let headers = request
            .headers
            .iter()
            .filter(|(name, _)| !is_restricted_webview_header(name))
            .cloned()
            .collect::<Vec<_>>();

        let wait_for = match self.kind {
            BrowserChallengeKind::Cloudflare => browser::WebViewWait::Script {
                // This SDK-owned expression waits for the exact challenge DOM
                // markers used by detection to disappear. It does not execute
                // downloaded extension code or expose a native bridge to the
                // page.
                script: r#"document.readyState === "complete" &&
                    !document.getElementById("challenge-error-title") &&
                    !document.getElementById("challenge-error-text")"#
                    .to_string(),
            },
        };
        browser::WebViewRequest {
            url: request.url.clone(),
            cookie_url: Some(self.cookie_url.clone()),
            session: Some(browser::WebViewSession {
                id: self.profile.clone(),
                ..browser::WebViewSession::default()
            }),
            wait_for: Some(wait_for),
            wait_until: Some(browser::WebViewWaitUntil::LoadFinished),
            user_agent,
            headers,
            timeout_ms: Some(self.timeout_ms),
            ..browser::WebViewRequest::default()
        }
    }

    fn resolve(&self, request: &net::Request) -> Result<()> {
        let _: browser::WebViewResponse = browser::open(&self.webview_request(request))?;
        Ok(())
    }
}

/// One RFC 7578 multipart/form-data part. This is the portable replacement
/// for common browser form and multipart API requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultipartPart {
    pub name: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

impl MultipartPart {
    pub fn text(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            filename: None,
            content_type: Some("text/plain; charset=utf-8".to_string()),
            bytes: value.into().into_bytes(),
        }
    }

    pub fn file(
        name: impl Into<String>,
        filename: impl Into<String>,
        content_type: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            name: name.into(),
            filename: Some(filename.into()),
            content_type: Some(content_type.into()),
            bytes: bytes.into(),
        }
    }
}

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
            .header("User-Agent", BROWSER_USER_AGENT)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/json;q=0.9,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        self.headers
            .retain(|(existing, _)| !existing.eq_ignore_ascii_case(&name));
        self.headers.push((name, value.into()));
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

#[derive(Clone)]
pub struct RequestBuilder {
    request: net::Request,
}

impl RequestBuilder {
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        self.request
            .headers
            .retain(|(existing, _)| !existing.eq_ignore_ascii_case(&name));
        self.request.headers.push((name, value.into()));
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

    /// Encode a bounded multipart request inside the guest and send it through
    /// the normal permission-checked host HTTP capability.
    pub fn multipart(self, parts: &[MultipartPart]) -> Result<Self> {
        if parts.len() > 256 {
            return Err(Error::new("multipart request exceeds 256 parts"));
        }
        let random = crate::runtime::random_bytes(16)?;
        let boundary = format!(
            "manatan-{}",
            random
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        let mut body = Vec::new();
        for part in parts {
            validate_multipart_token("name", &part.name)?;
            if let Some(filename) = part.filename.as_deref() {
                validate_multipart_token("filename", filename)?;
            }
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"", part.name).as_bytes(),
            );
            if let Some(filename) = part.filename.as_deref() {
                body.extend_from_slice(format!("; filename=\"{filename}\"").as_bytes());
            }
            body.extend_from_slice(b"\r\n");
            if let Some(content_type) = part.content_type.as_deref() {
                validate_multipart_header_value(content_type)?;
                body.extend_from_slice(format!("Content-Type: {content_type}\r\n").as_bytes());
            }
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(&part.bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        Ok(self
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body))
    }

    pub fn send(self) -> Result<Response> {
        net::fetch(&self.request).map(Response).map_err(Error::new)
    }

    /// Send normally, resolve a recognized browser challenge if necessary,
    /// then retry the same request once. The request automatically opts into
    /// host-managed cookies for the policy's origin.
    pub fn send_with_challenge(mut self, policy: &BrowserChallengePolicy) -> Result<Response> {
        if self.request.cookie_url.is_none() {
            self.request.cookie_url = Some(policy.cookie_url.clone());
        }
        ensure_browser_user_agent(&mut self.request);
        let first = net::fetch(&self.request)
            .map(Response)
            .map_err(Error::new)?;
        if !policy.matches(&first) {
            return Ok(first);
        }

        policy.resolve(&self.request)?;
        let retried = net::fetch(&self.request)
            .map(Response)
            .map_err(Error::new)?;
        if policy.matches(&retried) {
            return Err(Error::new(
                "browser challenge remained after one resolution attempt",
            ));
        }
        Ok(retried)
    }
}

fn ensure_browser_user_agent(request: &mut net::Request) {
    if !request
        .headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("user-agent"))
    {
        request
            .headers
            .push(("User-Agent".to_string(), BROWSER_USER_AGENT.to_string()));
    }
}

fn is_restricted_webview_header(name: &str) -> bool {
    [
        "connection",
        "content-length",
        "cookie",
        "host",
        "transfer-encoding",
        "user-agent",
    ]
    .iter()
    .any(|restricted| name.eq_ignore_ascii_case(restricted))
}

fn is_cloudflare_challenge(response: &Response) -> bool {
    if !matches!(response.status(), 403 | 503) {
        return false;
    }
    let cloudflare_server = response.header("server").is_some_and(|server| {
        server.eq_ignore_ascii_case("cloudflare") || server.eq_ignore_ascii_case("cloudflare-nginx")
    });
    if !cloudflare_server {
        return false;
    }
    let body = String::from_utf8_lossy(response.bytes());
    body.contains("challenge-error-title") || body.contains("challenge-error-text")
}

fn validate_multipart_token(kind: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| matches!(byte, b'\r' | b'\n' | b'"'))
    {
        return Err(Error::new(format!("invalid multipart {kind}")));
    }
    Ok(())
}

fn validate_multipart_header_value(value: &str) -> Result<()> {
    if value.is_empty() || value.bytes().any(|byte| matches!(byte, b'\r' | b'\n')) {
        return Err(Error::new("invalid multipart content type"));
    }
    Ok(())
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

    pub fn header(&self, name: &str) -> Option<&str> {
        self.0
            .headers
            .iter()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
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

#[cfg(test)]
mod tests {
    use super::{ensure_browser_user_agent, BrowserChallengePolicy, Client, Response};
    use crate::net;

    fn response(status: u16, server: &str, body: &str) -> Response {
        Response(net::Response {
            status,
            headers: vec![("Server".to_string(), server.to_string())],
            final_url: "https://example.com/api".to_string(),
            body: body.as_bytes().to_vec(),
        })
    }

    #[test]
    fn client_headers_override_case_insensitively() {
        let client = Client::browser().header("accept", "application/json");
        let accepts = client
            .headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("accept"))
            .collect::<Vec<_>>();
        assert_eq!(accepts.len(), 1);
        assert_eq!(accepts[0].1, "application/json");
    }

    #[test]
    fn request_headers_override_client_defaults() {
        let request = Client::browser()
            .get("https://example.com")
            .header("ACCEPT", "application/json");
        let accepts = request
            .request
            .headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("accept"))
            .collect::<Vec<_>>();
        assert_eq!(accepts.len(), 1);
        assert_eq!(accepts[0].1, "application/json");
    }

    #[test]
    fn cloudflare_policy_matches_only_recognized_challenge_pages() {
        let policy = BrowserChallengePolicy::cloudflare("https://example.com");
        assert!(policy.matches(&response(
            403,
            "cloudflare",
            r#"<h1 id="challenge-error-title">Please wait</h1>"#,
        )));
        assert!(policy.matches(&response(
            503,
            "cloudflare-nginx",
            r#"<div id='challenge-error-text'>Checking your browser</div>"#,
        )));
    }

    #[test]
    fn cloudflare_policy_does_not_treat_geo_blocks_or_normal_pages_as_challenges() {
        let policy = BrowserChallengePolicy::cloudflare("https://example.com");
        assert!(!policy.matches(&response(403, "cloudflare", "region blocked")));
        assert!(!policy.matches(&response(
            403,
            "origin",
            r#"<h1 id="challenge-error-title">Please wait</h1>"#,
        )));
        assert!(!policy.matches(&response(
            200,
            "cloudflare",
            r#"<script src="/cdn-cgi/challenge-platform/main.js"></script>"#,
        )));
    }

    #[test]
    fn cloudflare_policy_builds_a_scoped_persistent_webview_request() {
        let policy = BrowserChallengePolicy::cloudflare("https://example.com")
            .profile("source-cloudflare")
            .timeout_ms(12_345);
        let request = Client::browser()
            .header("Referer", "https://example.com/")
            .header("Cookie", "must-not-cross-the-host-boundary")
            .get("https://example.com/api/items")
            .request;
        let webview = policy.webview_request(&request);

        assert_eq!(webview.url, "https://example.com/api/items");
        assert_eq!(webview.cookie_url.as_deref(), Some("https://example.com"));
        assert_eq!(webview.timeout_ms, Some(12_345));
        assert_eq!(
            webview.user_agent.as_deref(),
            Some(super::BROWSER_USER_AGENT)
        );
        assert_eq!(
            webview.session.as_ref().map(|session| session.id.as_str()),
            Some("source-cloudflare")
        );
        assert!(webview.headers.iter().any(|(name, value)| {
            name.eq_ignore_ascii_case("referer") && value == "https://example.com/"
        }));
        assert!(!webview
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("cookie")));
        assert!(matches!(
            webview.wait_for,
            Some(crate::browser::WebViewWait::Script { .. })
        ));
    }

    #[test]
    fn challenge_requests_use_the_same_explicit_user_agent_in_http_and_webview() {
        let policy = BrowserChallengePolicy::cloudflare("https://example.com");
        let mut request = Client::new().get("https://example.com/api/items").request;
        ensure_browser_user_agent(&mut request);
        let webview = policy.webview_request(&request);

        let http_user_agent = request
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("user-agent"))
            .map(|(_, value)| value.as_str());
        assert_eq!(http_user_agent, Some(super::BROWSER_USER_AGENT));
        assert_eq!(webview.user_agent.as_deref(), http_user_agent);
    }
}
