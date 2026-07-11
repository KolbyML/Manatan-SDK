pub use scraper::{ElementRef, Html, Selector};

use crate::{Error, Result};

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
