//! Typed host-owned novel resources.

use crate::{ImageRequest, NovelResource, Result};

pub const TORRENT_EPUB_RESOURCE_VERSION: u32 = 1;
pub const TORRENT_EPUB_CAPABILITY: &str = "novel.resource.torrent-epub.v1";
pub const TORRENT_METAINFO_SERVICE: &str = "torrent.metainfo.v1";
pub const TORRENT_EPUB_COVER_SERVICE: &str = "torrent.epub.cover.v1";
pub const TORRENT_EPUB_COVER_URL: &str = "manatan-resource://torrent-epub-cover";

impl NovelResource {
    /// Build the host-mediated image request for an EPUB's embedded cover.
    /// Hosts return `404` until the exact EPUB has been materialized locally;
    /// browsing a catalogue therefore never downloads book payloads.
    pub fn epub_cover_request(&self) -> Result<ImageRequest> {
        Ok(ImageRequest::post(
            TORRENT_EPUB_COVER_URL,
            serde_json::to_vec(self)?,
        ))
    }
}
