use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{webview, Error, Result};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewRequest {
    pub url: String,
    #[serde(default)]
    pub cookie_url: Option<String>,
    #[serde(default)]
    pub wait_for: Option<WebViewWait>,
    #[serde(default)]
    pub wait_until: Option<WebViewWaitUntil>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub preload_scripts: Vec<WebViewScript>,
    #[serde(default)]
    pub scripts: Vec<WebViewScript>,
    #[serde(default)]
    pub return_html: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewExtractRequest {
    pub url: String,
    #[serde(default)]
    pub cookie_url: Option<String>,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub wait_until: Option<WebViewWaitUntil>,
    #[serde(default)]
    pub wait_for_script: Option<String>,
    #[serde(default)]
    pub wait_for_selector: Option<String>,
    #[serde(default)]
    pub wait_for_event: Option<String>,
    pub script: String,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub cookies: bool,
    #[serde(default)]
    pub headless: Option<bool>,
    #[serde(default)]
    pub preload_scripts: Vec<WebViewScript>,
    #[serde(default)]
    pub capture_requests: Vec<WebViewRequestCapture>,
    #[serde(default)]
    pub capture_events: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewExtractResponse {
    pub final_url: String,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub json: Option<Value>,
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub cookies: Vec<WebViewCookie>,
    #[serde(default)]
    pub captured_requests: Vec<WebViewCapturedRequest>,
    #[serde(default)]
    pub captured_events: Vec<WebViewCapturedEvent>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewResponse {
    pub final_url: String,
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub cookies: Vec<WebViewCookie>,
    #[serde(default)]
    pub captured_requests: Vec<WebViewCapturedRequest>,
    #[serde(default)]
    pub captured_events: Vec<WebViewCapturedEvent>,
    #[serde(default)]
    pub script_results: Vec<WebViewScriptResult>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub secure: Option<bool>,
    #[serde(default)]
    pub http_only: Option<bool>,
    #[serde(default)]
    pub expires_at: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewRequestCapture {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub url_contains: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub main_frame: Option<bool>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewCapturedRequest {
    #[serde(default)]
    pub capture_id: Option<String>,
    pub url: String,
    pub method: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub is_main_frame: bool,
    #[serde(default)]
    pub is_redirect: bool,
    #[serde(default)]
    pub frame_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewCapturedEvent {
    pub name: String,
    #[serde(default)]
    pub value: Option<Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WebViewWait {
    #[default]
    Load,
    Selector {
        selector: String,
    },
    UrlContains {
        value: String,
    },
    Script {
        script: String,
    },
    Delay {
        milliseconds: u64,
    },
    Event {
        name: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewScript {
    #[serde(default)]
    pub id: Option<String>,
    pub script: String,
    #[serde(default)]
    pub run_at: Option<WebViewScriptRunAt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebViewScriptRunAt {
    DocumentStart,
    DocumentEnd,
    AfterWait,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewScriptResult {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebViewWaitUntil {
    LoadStarted,
    #[default]
    LoadFinished,
    DomReady,
    NetworkIdle,
}

pub fn open<I, O>(request: &I) -> Result<O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    let request = serde_json::to_string(request)?;
    let response = webview::open(&request).map_err(Error::new)?;
    serde_json::from_str(&response).map_err(Error::from)
}

pub fn extract<I, O>(request: &I) -> Result<O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    let request = serde_json::to_string(request)?;
    let response = webview::extract(&request).map_err(Error::new)?;
    serde_json::from_str(&response).map_err(Error::from)
}
