use serde::{de::DeserializeOwned, Serialize};

use crate::{store, Error, Result};

pub fn get<T: DeserializeOwned>(namespace: &str, key: &str) -> Result<Option<T>> {
    store::get(namespace, key)
        .map_err(Error::new)?
        .map(|json| serde_json::from_str(&json).map_err(Error::from))
        .transpose()
}

pub fn set<T: Serialize>(namespace: &str, key: &str, value: &T) -> Result<()> {
    let json = serde_json::to_string(value)?;
    store::set(namespace, key, &json).map_err(Error::new)
}

pub fn delete(namespace: &str, key: &str) -> Result<()> {
    store::delete(namespace, key).map_err(Error::new)
}

pub fn list<T: DeserializeOwned>(namespace: &str) -> Result<Vec<(String, T)>> {
    store::list_entries(namespace)
        .map_err(Error::new)?
        .into_iter()
        .map(|(key, json)| {
            serde_json::from_str(&json)
                .map(|value| (key, value))
                .map_err(Error::from)
        })
        .collect()
}
