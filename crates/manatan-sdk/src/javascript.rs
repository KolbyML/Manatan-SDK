//! Bounded, host-provided ECMAScript evaluation for deobfuscation and token
//! generation. Only package assets can be executed: arbitrary or downloaded
//! script strings are deliberately not part of the SDK contract.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{services, Result};

pub const SERVICE: &str = "javascript.asset.v1";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateRequest {
    /// Declared package asset paths, evaluated in order. Assets are verified
    /// when the `.manatan2` package is installed.
    pub assets: Vec<String>,
    #[serde(default)]
    pub invocation: Option<Invocation>,
    #[serde(default)]
    pub globals: Value,
    #[serde(default)]
    pub loop_iteration_limit: Option<u64>,
}

/// Optional, data-only operation performed after the package assets have
/// loaded. Paths contain JavaScript identifier segments such as
/// `CryptoJS.AES.decrypt`; callers cannot submit source text for evaluation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Invocation {
    Call {
        function: String,
        #[serde(default)]
        arguments: Vec<Value>,
    },
    Read {
        path: String,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invocation_is_data_not_source_text() {
        let request = EvaluateRequest {
            assets: vec!["assets/token.js".to_string()],
            invocation: Some(Invocation::Call {
                function: "Token.create".to_string(),
                arguments: vec![serde_json::json!({ "episode": "one" })],
            }),
            globals: Value::Null,
            loop_iteration_limit: Some(100_000),
        };
        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["invocation"]["type"], "call");
        assert_eq!(value["invocation"]["function"], "Token.create");
        assert!(value.get("expression").is_none());
        assert!(value.get("scripts").is_none());
    }
}
