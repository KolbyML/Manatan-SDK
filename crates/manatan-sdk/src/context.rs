//! Per-call source context supplied by the Manatan host.

use std::cell::RefCell;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{Error, Result};

#[derive(Default)]
struct CallContext {
    source_id: Option<String>,
    preferences: Value,
}

thread_local! {
    static CONTEXT: RefCell<CallContext> = RefCell::new(CallContext::default());
}

/// Id of the source handling the current host call.
pub fn source_id() -> Option<String> {
    CONTEXT.with(|context| context.borrow().source_id.clone())
}

/// Current source preference payload, including host-applied values.
pub fn preferences() -> Value {
    CONTEXT.with(|context| context.borrow().preferences.clone())
}

/// Look up a preference in either an object payload or Manatan's preference
/// definition list shape.
pub fn preference_value(key: &str) -> Option<Value> {
    CONTEXT.with(|context| {
        let context = context.borrow();
        match &context.preferences {
            Value::Object(values) => values.get(key).cloned(),
            Value::Array(values) => values.iter().find_map(|preference| {
                let matches = preference
                    .get("key")
                    .or_else(|| preference.get("id"))
                    .and_then(Value::as_str)
                    == Some(key);
                matches.then(|| {
                    preference
                        .get("value")
                        .or_else(|| preference.get("currentValue"))
                        .or_else(|| preference.get("default"))
                        .cloned()
                        .unwrap_or(Value::Null)
                })
            }),
            _ => None,
        }
    })
}

/// Deserialize a preference value into an extension-defined type.
pub fn preference<T: DeserializeOwned>(key: &str) -> Result<Option<T>> {
    preference_value(key)
        .map(|value| serde_json::from_value(value).map_err(Error::from))
        .transpose()
}

pub(crate) fn enter(source_id: &str, preferences: Value) -> CallContextGuard {
    let previous = CONTEXT.with(|context| {
        std::mem::replace(
            &mut *context.borrow_mut(),
            CallContext {
                source_id: Some(source_id.to_string()),
                preferences,
            },
        )
    });
    CallContextGuard(Some(previous))
}

pub(crate) struct CallContextGuard(Option<CallContext>);

impl Drop for CallContextGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.0.take() {
            CONTEXT.with(|context| *context.borrow_mut() = previous);
        }
    }
}
