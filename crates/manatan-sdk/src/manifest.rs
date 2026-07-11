use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 2;
pub const API_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub version_code: i64,
    pub api_version: u32,
    pub wasm: String,
    pub content_type: ContentType,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub minimum_manatan_version: Option<String>,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub assets: Vec<Asset>,
    pub sources: Vec<Source>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Manga,
    Video,
    Novel,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    #[serde(default)]
    pub network: NetworkPermission,
    #[serde(default)]
    pub webview: bool,
    #[serde(default)]
    pub cookies: bool,
    #[serde(default)]
    pub storage: bool,
    #[serde(default)]
    pub assets: bool,
    #[serde(default)]
    pub javascript: bool,
    /// Additional forward-compatible host service names or wildcard prefixes.
    #[serde(default)]
    pub services: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkPermission {
    #[serde(default)]
    pub allow: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub path: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: String,
    pub name: String,
    pub lang: String,
    pub content_type: ContentType,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub content_rating: ContentRating,
    #[serde(default)]
    pub capabilities: Capabilities,
    #[serde(default)]
    pub listings: Vec<Listing>,
    #[serde(default)]
    pub url_patterns: Vec<UrlPattern>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentRating {
    Safe,
    Suggestive,
    Adult,
    #[default]
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    /// Advertised namespaced operations handled by the source `dispatch`
    /// hook, for example `commands.describe` or `comments.list`.
    #[serde(default)]
    pub operations: Vec<String>,
    #[serde(default)]
    pub search: bool,
    #[serde(default)]
    pub latest: bool,
    #[serde(default)]
    pub filters: bool,
    #[serde(default)]
    pub preferences: bool,
    #[serde(default)]
    pub home: bool,
    #[serde(default)]
    pub hoster_resolution: bool,
    #[serde(default)]
    pub seasons: bool,
    #[serde(default)]
    pub url_resolution: bool,
    #[serde(default)]
    pub chapter_pagination: bool,
    #[serde(default)]
    pub page_processing: bool,
    #[serde(default)]
    pub media_processing: bool,
    #[serde(default)]
    pub authentication: bool,
    #[serde(default)]
    pub unmetered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listing {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlPattern {
    pub pattern: String,
    #[serde(default)]
    pub kind: Option<String>,
}

impl Manifest {
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(format!("schemaVersion must be {SCHEMA_VERSION}"));
        }
        if self.api_version != API_VERSION {
            return Err(format!("apiVersion must be {API_VERSION}"));
        }
        if !valid_id(&self.id) {
            return Err("id must start with a lowercase letter and contain only lowercase letters, digits, '.', '_', or '-'".into());
        }
        if self.sources.is_empty() {
            return Err("at least one source is required".into());
        }
        if !self.wasm.ends_with(".wasm") || self.wasm.contains("..") || self.wasm.starts_with('/') {
            return Err("wasm must be a safe relative .wasm path".into());
        }
        if let Some(icon) = self.icon.as_deref() {
            if !safe_relative_path(icon) || icon == "manifest.json" || icon == self.wasm {
                return Err("icon must be a safe relative package path".into());
            }
        }
        let mut ids = std::collections::BTreeSet::new();
        for source in &self.sources {
            if !valid_id(&source.id) {
                return Err(format!("invalid source id {:?}", source.id));
            }
            if source.content_type != self.content_type {
                return Err(format!("source {:?} has the wrong contentType", source.id));
            }
            if !ids.insert(source.id.as_str()) {
                return Err(format!("duplicate source id {:?}", source.id));
            }
        }
        let mut asset_paths = std::collections::BTreeSet::new();
        for asset in &self.assets {
            if !safe_relative_path(&asset.path) {
                return Err(format!("invalid asset path {:?}", asset.path));
            }
            if !asset_paths.insert(asset.path.as_str()) {
                return Err(format!("duplicate asset path {:?}", asset.path));
            }
            if let Some(digest) = asset.sha256.as_deref() {
                if digest.len() != 64 || !digest.chars().all(|ch| ch.is_ascii_hexdigit()) {
                    return Err(format!("invalid SHA-256 for asset {:?}", asset.path));
                }
            }
        }
        Ok(())
    }
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.starts_with('\\')
        && !value
            .split(['/', '\\'])
            .any(|part| part.is_empty() || part == "." || part == "..")
}

fn valid_id(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && chars.all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-')
        })
}
