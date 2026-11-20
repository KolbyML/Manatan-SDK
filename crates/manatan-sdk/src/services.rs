//! Forward-compatible host service dispatch.

use serde::{de::DeserializeOwned, Serialize};

use crate::{service_host, Error, Result};

pub fn available() -> Vec<String> {
    service_host::available()
}

pub fn is_available(service: &str) -> bool {
    available().iter().any(|value| value == service)
}

pub fn invoke<I, O>(service: &str, request: &I) -> Result<O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    let request = serde_json::to_string(request)?;
    let response = service_host::invoke(service, &request).map_err(Error::new)?;
    serde_json::from_str(&response).map_err(Error::from)
}

pub fn invoke_binary<I, O>(service: &str, request: &I, bytes: &[u8]) -> Result<(O, Vec<u8>)>
where
    I: Serialize,
    O: DeserializeOwned,
{
    let request = serde_json::to_string(request)?;
    let response = service_host::invoke_binary(service, &request, bytes).map_err(Error::new)?;
    let metadata = serde_json::from_str(&response.response_json).map_err(Error::from)?;
    Ok((metadata, response.bytes))
}

/// A parsed torrent and its metainfo file indices.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentMetainfo {
    pub name: Option<String>,
    pub info_hash: String,
    pub files: Vec<TorrentFile>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TorrentFile {
    pub index: usize,
    pub path: String,
    pub length: u64,
}

pub const TORRENT_METAINFO: &str = "torrent.metainfo.v1";

/// Inspect downloaded metainfo without starting a torrent session.
pub fn inspect_torrent(
    metainfo: &[u8],
    expected_info_hash: Option<&str>,
) -> Result<TorrentMetainfo> {
    let request = serde_json::json!({ "expectedInfoHash": expected_info_hash });
    let (result, _) = invoke_binary(TORRENT_METAINFO, &request, metainfo)?;
    Ok(result)
}
