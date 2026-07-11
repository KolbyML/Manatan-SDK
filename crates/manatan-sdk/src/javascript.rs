//! Bounded, host-provided ECMAScript evaluation for deobfuscation and token
//! generation. It has no filesystem or network globals.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{services, Result};

pub const SERVICE: &str = "javascript.eval";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateRequest {
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub expression: Option<String>,
    #[serde(default)]
    pub globals: Value,
    #[serde(default)]
    pub loop_iteration_limit: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponse {
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub text: Option<String>,
}

pub fn evaluate(request: &EvaluateRequest) -> Result<EvaluateResponse> {
    services::invoke(SERVICE, request)
}
