use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type Headers = BTreeMap<String, String>;
pub type Extra = BTreeMap<String, Value>;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paged<T> {
    #[serde(default)]
    pub entries: Vec<T>,
    #[serde(default)]
    pub has_next_page: bool,
}

impl<T> Paged<T> {
    pub fn new(entries: Vec<T>, has_next_page: bool) -> Self {
        Self {
            entries,
            has_next_page,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItem {
    pub key: String,
    pub title: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub cover: Option<ImageRequest>,
    #[serde(default)]
    pub banner: Option<ImageRequest>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub artists: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub status: Option<Value>,
    #[serde(default)]
    pub initialized: bool,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub rating: Option<f32>,
    #[serde(default)]
    pub content_rating: Option<String>,
    #[serde(default)]
    pub viewer: Option<Value>,
    #[serde(default)]
    pub update_strategy: Option<Value>,
    #[serde(default)]
    pub fetch_type: Option<String>,
    #[serde(default)]
    pub season_number: Option<f64>,
    #[serde(default)]
    pub next_update_time: Option<i64>,
    #[serde(default)]
    pub alternate_covers: Vec<AlternateCover>,
    #[serde(default)]
    pub extra: Extra,
}

impl CatalogItem {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlternateCover {
    pub image: ImageRequest,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaChapter {
    pub key: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub chapter_number: Option<f32>,
    #[serde(default)]
    pub volume_number: Option<f32>,
    #[serde(default)]
    pub date_uploaded: Option<i64>,
    #[serde(default)]
    pub scanlators: Vec<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<ImageRequest>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub source_order: Option<i32>,
    #[serde(default)]
    pub is_locked: bool,
    #[serde(default)]
    pub page_count: Option<u32>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaPage {
    pub content: PageContent,
    #[serde(default)]
    pub thumbnail: Option<ImageRequest>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub headers: Headers,
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PageContent {
    Url {
        url: String,
        #[serde(default)]
        context: Option<Headers>,
    },
    Text {
        text: String,
    },
    Request(ImageRequest),
    Inline {
        data_base64: String,
        #[serde(default)]
        mime_type: Option<String>,
    },
    Lazy {
        key: String,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        page_url: Option<String>,
        #[serde(default)]
        context: Option<Headers>,
    },
}

impl Default for PageContent {
    fn default() -> Self {
        Self::Text {
            text: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRequest {
    pub url: String,
    /// URL whose scoped cookie jar should be attached by the host. Raw Cookie
    /// headers are intentionally unsupported.
    #[serde(default)]
    pub cookie_url: Option<String>,
    #[serde(default)]
    pub method: ImageRequestMethod,
    #[serde(default)]
    pub headers: Headers,
    #[serde(default)]
    pub body: Option<Vec<u8>>,
}

impl ImageRequest {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: ImageRequestMethod::Get,
            ..Self::default()
        }
    }

    pub fn post(url: impl Into<String>, body: impl Into<Vec<u8>>) -> Self {
        Self {
            url: url.into(),
            method: ImageRequestMethod::Post,
            body: Some(body.into()),
            ..Self::default()
        }
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn cookies_for(mut self, url: impl Into<String>) -> Self {
        self.cookie_url = Some(url.into());
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageRequestMethod {
    #[default]
    Get,
    Post,
}

impl From<String> for ImageRequest {
    fn from(url: String) -> Self {
        Self::get(url)
    }
}

impl From<&str> for ImageRequest {
    fn from(url: &str) -> Self {
        Self::get(url)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaPageImage {
    pub url: String,
    #[serde(default)]
    pub headers: Headers,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedImage {
    #[serde(default)]
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub mime_type: Option<String>,
}

/// A binary media resource returned after guest-side processing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedMedia {
    #[serde(default)]
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub mime_type: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoEpisode {
    pub key: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub episode_number: Option<f32>,
    #[serde(default)]
    pub season_number: Option<f32>,
    #[serde(default)]
    pub date_uploaded: Option<i64>,
    #[serde(default)]
    pub thumbnail: Option<ImageRequest>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub duration_seconds: Option<f64>,
    #[serde(default)]
    pub release_group: Option<String>,
    #[serde(default)]
    pub variant: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub is_filler: bool,
    #[serde(default)]
    pub is_locked: bool,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoHoster {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub lazy: bool,
    #[serde(default)]
    pub video_count: Option<u32>,
    #[serde(default)]
    pub internal_data: Option<String>,
    #[serde(default)]
    pub resolved_streams: Vec<VideoStream>,
    #[serde(default)]
    pub headers: Headers,
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStream {
    pub url: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hoster: Option<VideoHoster>,
    #[serde(default)]
    pub quality: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub bitrate: Option<u64>,
    #[serde(default)]
    pub video_codec: Option<String>,
    #[serde(default)]
    pub audio_codec: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub duration_seconds: Option<f64>,
    #[serde(default)]
    pub is_hls: bool,
    #[serde(default)]
    pub is_dash: bool,
    #[serde(default)]
    pub is_backup: bool,
    #[serde(default)]
    pub requires_proxy: bool,
    #[serde(default)]
    pub preferred: bool,
    #[serde(default)]
    pub initialized: bool,
    #[serde(default)]
    pub headers: Headers,
    #[serde(default)]
    pub audio_tracks: Vec<MediaTrack>,
    #[serde(default)]
    pub subtitles: Vec<MediaTrack>,
    #[serde(default)]
    pub intro: Option<MediaSegment>,
    #[serde(default)]
    pub outro: Option<MediaSegment>,
    #[serde(default)]
    pub timestamps: Vec<MediaTimestamp>,
    #[serde(default)]
    pub drm: Option<DrmInfo>,
    #[serde(default)]
    pub mpv_args: Vec<PlayerArg>,
    #[serde(default)]
    pub ffmpeg_stream_args: Vec<PlayerArg>,
    #[serde(default)]
    pub ffmpeg_video_args: Vec<PlayerArg>,
    #[serde(default)]
    pub internal_data: Option<String>,
    #[serde(default)]
    pub stream_kind: Option<String>,
    #[serde(default)]
    pub torrent: Option<TorrentInfo>,
    #[serde(default)]
    pub debrid: Option<DebridInfo>,
    #[serde(default)]
    pub segment_processing: Option<SegmentProcessing>,
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentInfo {
    #[serde(default)]
    pub magnet_uri: Option<String>,
    #[serde(default)]
    pub info_hash: Option<String>,
    #[serde(default)]
    pub file_index: Option<i32>,
    #[serde(default)]
    pub trackers: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebridInfo {
    pub provider: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub account_required: bool,
    #[serde(default)]
    pub extra: Extra,
}

/// Host-mediated playlist and media resource processing. Rules are combined
/// in declaration order; matching fixed-prefix skips are summed and automatic
/// media detection runs once after those skips.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentProcessing {
    #[serde(default)]
    pub rewrite_playlist: bool,
    /// Attach cookies applicable to each proxied resource URL from the
    /// source-scoped host jar. Raw or cross-origin Cookie headers cannot be
    /// embedded in stream metadata.
    #[serde(default)]
    pub cookies: bool,
    #[serde(default)]
    pub guest_transform: bool,
    #[serde(default)]
    pub max_resource_bytes: Option<u64>,
    #[serde(default)]
    pub rules: Vec<SegmentRule>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentRule {
    #[serde(default)]
    pub resource_types: Vec<MediaResourceKind>,
    #[serde(default)]
    pub url_contains: Vec<String>,
    #[serde(default)]
    pub host_patterns: Vec<String>,
    /// Headers merged only for resources matching this rule. This replaces
    /// per-route OkHttp interceptors and local NanoHTTPD forwarding servers.
    #[serde(default)]
    pub headers: Headers,
    #[serde(default)]
    pub strip_prefix_bytes: Option<u64>,
    #[serde(default)]
    pub auto_detect_media_offset: bool,
    #[serde(default)]
    pub probe_bytes: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaResourceKind {
    Playlist,
    Segment,
    Key,
    Subtitle,
    Audio,
    Video,
    Other,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTrack {
    pub url: String,
    #[serde(default)]
    pub inline_data: Option<String>,
    #[serde(default)]
    pub inline_base64: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub headers: Headers,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub is_forced: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSegment {
    pub start_seconds: f64,
    pub end_seconds: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTimestamp {
    pub time_seconds: f64,
    /// Optional end of a ranged marker. When absent, the marker represents a
    /// single point in the media timeline.
    #[serde(default)]
    pub end_seconds: Option<f64>,
    pub label: String,
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerArg {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrmInfo {
    pub scheme: String,
    #[serde(default)]
    pub license_url: Option<String>,
    #[serde(default)]
    pub headers: Headers,
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelChapter {
    pub key: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub chapter_number: Option<f32>,
    #[serde(default)]
    pub volume_number: Option<f32>,
    #[serde(default)]
    pub date_uploaded: Option<i64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub source_order: Option<i32>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub release_group: Option<String>,
    #[serde(default)]
    pub word_count: Option<u32>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub is_locked: bool,
    /// Host-owned binary content used to materialize this chapter. Extensions
    /// describe the immutable resource; they never download or parse it.
    #[serde(default)]
    pub resource: Option<NovelResource>,
    #[serde(default)]
    pub extra: Extra,
}

/// A versioned, host-materialized resource backing novel content.
///
/// Resource variants are intentionally typed instead of being placed in
/// `extra`, so hosts can validate permissions, identity, and platform support
/// before doing native work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum NovelResource {
    TorrentEpub {
        /// Schema version for this descriptor. Version 1 is the only version
        /// currently defined.
        version: u32,
        /// A magnet URI or HTTPS URL ending in `.torrent`.
        uri: String,
        /// Immutable v1 BitTorrent info hash as 40 hexadecimal characters.
        info_hash: String,
        /// Exact zero-based file index in the torrent metadata.
        file_index: u32,
        /// Exact normalized torrent path. Hosts verify it against the selected
        /// file index before materializing the EPUB.
        file_path: String,
        /// Expected file length, when known from inspected torrent metadata.
        #[serde(default)]
        length_bytes: Option<u64>,
        /// Optional additional trackers for magnet resources.
        #[serde(default)]
        trackers: Vec<String>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelText {
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub css: Option<String>,
    /// Request defaults for URL images embedded in `html`. Typed image blocks
    /// carry their own complete `ImageRequest` instead.
    #[serde(default)]
    pub image_context: Option<ImageRequestContext>,
    #[serde(default)]
    pub next_chapter_key: Option<String>,
    #[serde(default)]
    pub previous_chapter_key: Option<String>,
    /// Host-owned resource to materialize when the text itself is deferred.
    /// This normally mirrors `NovelChapter::resource`.
    #[serde(default)]
    pub resource: Option<NovelResource>,
    #[serde(default)]
    pub blocks: Vec<NovelContentBlock>,
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRequestContext {
    #[serde(default)]
    pub headers: Headers,
    #[serde(default)]
    pub cookie_url: Option<String>,
}

/// Ordered mixed content used by compatible sources. `html`/`text`
/// remain available on `NovelText` for simple text-first readers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NovelContentBlock {
    Text {
        text: String,
        #[serde(default)]
        html: bool,
    },
    Image {
        image: ImageRequest,
        #[serde(default)]
        alt: Option<String>,
    },
    InlineImage {
        data_base64: String,
        #[serde(default)]
        mime_type: Option<String>,
        #[serde(default)]
        alt: Option<String>,
    },
    Video {
        url: String,
        #[serde(default)]
        headers: Headers,
    },
    Audio {
        url: String,
        #[serde(default)]
        headers: Headers,
    },
    Subtitle(MediaTrack),
    PageUrl {
        url: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelChapterPage {
    #[serde(default)]
    pub entries: Vec<NovelChapter>,
    #[serde(default)]
    pub has_next_page: bool,
    #[serde(default)]
    pub page_count: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeSection {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub entries: Vec<CatalogItem>,
    #[serde(default)]
    pub style: HomeSectionStyle,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HomeSectionStyle {
    #[default]
    Grid,
    List,
    Carousel,
    Featured,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FilterDefinition {
    Header {
        name: String,
    },
    Separator,
    Text {
        id: String,
        name: String,
        default: String,
    },
    CheckBox {
        id: String,
        name: String,
        default: bool,
    },
    TriState {
        id: String,
        name: String,
        #[serde(default)]
        default: i8,
    },
    Select {
        id: String,
        name: String,
        options: Vec<OptionItem>,
        default_index: u32,
    },
    MultiSelect {
        id: String,
        name: String,
        options: Vec<OptionItem>,
        #[serde(default)]
        default: Vec<String>,
    },
    Sort {
        id: String,
        name: String,
        options: Vec<SortOption>,
        #[serde(default)]
        default: Option<SortSelection>,
    },
    Group {
        id: String,
        name: String,
        filters: Vec<FilterDefinition>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionItem {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortOption {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortSelection {
    pub index: u32,
    #[serde(default)]
    pub ascending: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterValue {
    pub id: String,
    pub value: Value,
}

/// Media-neutral credentials for sources with explicit sign-in flows. The
/// values map permits username/password, API-key, device-code, two-factor, or
/// site-specific fields without changing the v2 wire contract.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationRequest {
    #[serde(default)]
    pub values: Extra,
    #[serde(default)]
    pub interactive: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationState {
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default)]
    pub account_name: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    /// Next user-visible step when authentication cannot finish in the
    /// current call. The host presents it and calls `authenticate` again with
    /// the collected values or after the browser step completes.
    #[serde(default)]
    pub action: Option<AuthenticationAction>,
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AuthenticationAction {
    Form {
        title: String,
        #[serde(default)]
        message: Option<String>,
        fields: Vec<AuthenticationField>,
        #[serde(default)]
        submit_label: Option<String>,
    },
    WebView {
        url: String,
        /// Source-local persistent browser profile used for this flow.
        #[serde(default = "default_auth_profile")]
        profile: String,
        #[serde(default)]
        cookie_url: Option<String>,
        #[serde(default)]
        completion: AuthenticationWebViewCompletion,
    },
    ExternalBrowser {
        url: String,
        #[serde(default)]
        callback_url: Option<String>,
    },
}

fn default_auth_profile() -> String {
    "login".to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationWebViewCompletion {
    /// URL patterns which allow the host to close the WebView automatically.
    #[serde(default)]
    pub close_url_patterns: Vec<String>,
    /// Cookie names that must exist for `cookie_url` before auto-close.
    #[serde(default)]
    pub required_cookie_names: Vec<String>,
    /// Challenge flows normally remain manually closable even without a
    /// deterministic success URL.
    #[serde(default = "default_true")]
    pub allow_manual_close: bool,
}

impl Default for AuthenticationWebViewCompletion {
    fn default() -> Self {
        Self {
            close_url_patterns: Vec::new(),
            required_cookie_names: Vec::new(),
            allow_manual_close: true,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationField {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub kind: AuthenticationFieldKind,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub options: Vec<OptionItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthenticationFieldKind {
    #[default]
    Text,
    Password,
    OneTimeCode,
    ApiKey,
    Select,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PreferenceDefinition {
    Text {
        key: String,
        title: String,
        #[serde(default)]
        summary: Option<String>,
        #[serde(default)]
        default: String,
        #[serde(default)]
        hint: Option<String>,
        #[serde(default)]
        secure: bool,
        #[serde(default)]
        multiline: bool,
    },
    Switch {
        key: String,
        title: String,
        #[serde(default)]
        summary: Option<String>,
        #[serde(default)]
        default: bool,
    },
    Select {
        key: String,
        title: String,
        options: Vec<OptionItem>,
        #[serde(default)]
        default: String,
    },
    MultiSelect {
        key: String,
        title: String,
        #[serde(default)]
        summary: Option<String>,
        options: Vec<OptionItem>,
        #[serde(default)]
        default: Vec<String>,
    },
    Number {
        key: String,
        title: String,
        #[serde(default)]
        summary: Option<String>,
        default: i64,
        min: i64,
        max: i64,
        #[serde(default = "default_number_step")]
        step: i64,
    },
    Info {
        title: String,
        #[serde(default)]
        summary: Option<String>,
    },
    Link {
        key: String,
        title: String,
        url: String,
        #[serde(default)]
        summary: Option<String>,
    },
    Group {
        title: String,
        preferences: Vec<PreferenceDefinition>,
    },
}

fn default_number_step() -> i64 {
    1
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlResolveResult {
    #[serde(default)]
    pub item: Option<CatalogItem>,
    #[serde(default)]
    pub chapter_key: Option<String>,
    #[serde(default)]
    pub episode_key: Option<String>,
    #[serde(default)]
    pub manga_chapter: Option<MangaChapter>,
    #[serde(default)]
    pub video_episode: Option<VideoEpisode>,
    #[serde(default)]
    pub novel_chapter: Option<NovelChapter>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListRequest {
    #[serde(default = "default_listing")]
    pub listing: String,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default)]
    pub filters: Value,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchRequest {
    #[serde(default)]
    pub query: String,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default)]
    pub filters: Value,
}

pub(crate) fn default_page() -> u32 {
    1
}

fn default_listing() -> String {
    "popular".to_string()
}
