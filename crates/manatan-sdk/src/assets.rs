//! Read inert files declared in the `.manatan2` manifest.

use crate::{asset_host, Error, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetInfo {
    pub path: String,
    pub size: u64,
    pub mime_type: Option<String>,
}

pub fn list() -> Result<Vec<AssetInfo>> {
    asset_host::list_assets()
        .map(|assets| {
            assets
                .into_iter()
                .map(|asset| AssetInfo {
                    path: asset.path,
                    size: asset.size,
                    mime_type: asset.mime_type,
                })
                .collect()
        })
        .map_err(Error::new)
}

pub fn read(path: &str) -> Result<Vec<u8>> {
    asset_host::read(path).map_err(Error::new)
}
