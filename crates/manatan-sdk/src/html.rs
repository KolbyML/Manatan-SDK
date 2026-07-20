use std::collections::BTreeMap;

pub use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{services, Error, Result};

pub const SELECT_SERVICE: &str = "html.select.v1";

/// A bounded batch of CSS selector queries executed by the native host.
///
/// Use this for large pages when parsing a complete DOM inside an interpreted
/// WebAssembly guest would be unnecessarily expensive. Extensions must declare
/// [`SELECT_SERVICE`] in `permissions.services`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectRequest {
    #[serde(default)]
    pub fragment: bool,
    pub queries: Vec<SelectQuery>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectQuery {
    pub id: String,
    pub selector: String,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub fields: Vec<SelectField>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectField {
    pub name: String,
    #[serde(default)]
    pub selector: Option<String>,
    pub value: SelectValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SelectValue {
    Text,
    InnerHtml,
    OuterHtml,
    Attribute { name: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectResponse {
    pub results: BTreeMap<String, Vec<BTreeMap<String, Option<String>>>>,
}

impl SelectQuery {
    pub fn new(id: impl Into<String>, selector: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            selector: selector.into(),
            limit: None,
            fields: Vec::new(),
        }
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn field(mut self, field: SelectField) -> Self {
        self.fields.push(field);
        self
    }
}

impl SelectField {
    pub fn new(name: impl Into<String>, value: SelectValue) -> Self {
        Self {
            name: name.into(),
            selector: None,
            value,
        }
    }

    pub fn within(mut self, selector: impl Into<String>) -> Self {
        self.selector = Some(selector.into());
        self
    }
}

pub fn native_selection_available() -> bool {
    services::is_available(SELECT_SERVICE)
}

/// Parse the input once in the native host and return all requested values.
pub fn select(request: &SelectRequest, bytes: &[u8]) -> Result<SelectResponse> {
    services::invoke_binary(SELECT_SERVICE, request, bytes).map(|(response, _)| response)
}

pub fn document(html: &str) -> Html {
    Html::parse_document(html)
}

pub fn fragment(html: &str) -> Html {
    Html::parse_fragment(html)
}

pub fn selector(value: &str) -> Result<Selector> {
    Selector::parse(value).map_err(|error| Error::new(error.to_string()))
}

pub fn text(element: ElementRef<'_>) -> String {
    element
        .text()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn attribute(element: ElementRef<'_>, name: &str) -> Option<String> {
    element.value().attr(name).map(ToOwned::to_owned)
}

pub fn absolute_url(base: &str, candidate: &str) -> Result<String> {
    let base = url::Url::parse(base).map_err(|error| Error::new(error.to_string()))?;
    base.join(candidate)
        .map(|url| url.to_string())
        .map_err(|error| Error::new(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_native_selection_contract() {
        let request = SelectRequest {
            fragment: false,
            queries: vec![SelectQuery::new("cards", "article")
                .limit(20)
                .field(SelectField::new("title", SelectValue::Text).within("a.title"))
                .field(
                    SelectField::new(
                        "href",
                        SelectValue::Attribute {
                            name: "href".to_string(),
                        },
                    )
                    .within("a.title"),
                )],
        };
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            serde_json::json!({
                "fragment": false,
                "queries": [{
                    "id": "cards",
                    "selector": "article",
                    "limit": 20,
                    "fields": [
                        {
                            "name": "title",
                            "selector": "a.title",
                            "value": { "type": "text" }
                        },
                        {
                            "name": "href",
                            "selector": "a.title",
                            "value": { "type": "attribute", "name": "href" }
                        }
                    ]
                }]
            })
        );
    }
}
