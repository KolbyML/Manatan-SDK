use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 2;
pub const API_VERSION: u32 = 2;
pub const PACKAGE_SIGNATURE_DOMAIN: &[u8] = b"manatan2-package-signature-v1\0";

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
    /// Stable package publisher identity and detached Ed25519 signature over
    /// the canonical manifest plus every non-manifest archive entry.
    pub publisher: Publisher,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    /// Stable reverse-DNS-style publisher identifier.
    pub id: String,
    /// Lowercase hexadecimal Ed25519 public key (32 bytes).
    pub public_key: String,
    /// Lowercase hexadecimal Ed25519 signature (64 bytes).
    pub signature: String,
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
    /// Permit bounded execution of SHA-256-pinned JavaScript package assets.
    /// This never permits arbitrary or downloaded script text.
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
        if !valid_id(&self.publisher.id) {
            return Err("publisher.id must use the package id character set".into());
        }
        if !valid_hex(&self.publisher.public_key, 64) {
            return Err("publisher.publicKey must be a 32-byte hexadecimal Ed25519 key".into());
        }
        if !valid_hex(&self.publisher.signature, 128) {
            return Err(
                "publisher.signature must be a 64-byte hexadecimal Ed25519 signature".into(),
            );
        }
        if self.sources.is_empty() {
            return Err("at least one source is required".into());
        }
        if !self.wasm.ends_with(".wasm") || self.wasm.contains("..") || self.wasm.starts_with('/') {
            return Err("wasm must be a safe relative .wasm path".into());
        }
        if let Some(icon) = self.icon.as_deref() {
            if !safe_relative_path(icon)
                || forbidden_executable_asset(icon)
                || !supported_icon_path(icon)
                || icon == "manifest.json"
                || icon == self.wasm
            {
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
            if source.content_rating == ContentRating::Unknown {
                return Err(format!(
                    "source {:?} must declare safe, suggestive, or adult contentRating",
                    source.id
                ));
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
            if forbidden_executable_asset(&asset.path) {
                return Err(format!(
                    "executable/native package asset {:?} is not allowed",
                    asset.path
                ));
            }
            if asset.path == "manifest.json" || asset.path == self.wasm {
                return Err(format!("invalid asset path {:?}", asset.path));
            }
            if javascript_asset(&asset.path) && asset.sha256.is_none() {
                return Err(format!(
                    "JavaScript package asset {:?} requires SHA-256",
                    asset.path
                ));
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

/// Build the deterministic byte payload signed by a package publisher.
///
/// `entry_digests` contains the SHA-256 of every non-directory archive entry
/// except `manifest.json`. Entry names are sorted by raw UTF-8 order. The
/// manifest's `publisher.signature` field is omitted from the signed JSON.
pub fn package_signature_payload(
    manifest_value: &serde_json::Value,
    entry_digests: &[(String, [u8; 32])],
) -> std::result::Result<Vec<u8>, String> {
    let mut unsigned_manifest = manifest_value.clone();
    unsigned_manifest
        .get_mut("publisher")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| "manifest publisher is missing".to_string())?
        .remove("signature");

    let mut payload = Vec::new();
    payload.extend_from_slice(PACKAGE_SIGNATURE_DOMAIN);
    write_canonical_json(&unsigned_manifest, &mut payload)?;
    let mut entries = entry_digests.iter().collect::<Vec<_>>();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
    for (name, digest) in entries {
        payload.extend_from_slice(&(name.len() as u64).to_be_bytes());
        payload.extend_from_slice(name.as_bytes());
        payload.extend_from_slice(digest);
    }
    Ok(payload)
}

fn write_canonical_json(
    value: &serde_json::Value,
    output: &mut Vec<u8>,
) -> std::result::Result<(), String> {
    match value {
        serde_json::Value::Null => output.extend_from_slice(b"null"),
        serde_json::Value::Bool(value) => {
            output.extend_from_slice(if *value { b"true" } else { b"false" })
        }
        serde_json::Value::Number(value) => output.extend_from_slice(value.to_string().as_bytes()),
        serde_json::Value::String(value) => {
            serde_json::to_writer(output, value).map_err(|error| error.to_string())?
        }
        serde_json::Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        serde_json::Value::Object(values) => {
            output.push(b'{');
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key).map_err(|error| error.to_string())?;
                output.push(b':');
                write_canonical_json(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn forbidden_executable_asset(path: &str) -> bool {
    path.rsplit_once('.').is_some_and(|(_, extension)| {
        matches!(
            extension.to_ascii_lowercase().as_str(),
            "apk" | "jar" | "dex" | "class" | "so" | "dylib" | "dll" | "exe" | "o" | "a" | "pyc"
        )
    })
}

fn javascript_asset(path: &str) -> bool {
    path.rsplit_once('.').is_some_and(|(_, extension)| {
        matches!(extension.to_ascii_lowercase().as_str(), "js" | "mjs")
    })
}

fn supported_icon_path(path: &str) -> bool {
    path.rsplit_once('.').is_some_and(|(_, extension)| {
        matches!(
            extension.to_ascii_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "avif"
        )
    })
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn signature_payload_is_canonical_and_omits_signature() {
        let left = json!({
            "z": 2,
            "publisher": { "signature": "ignored", "publicKey": "key", "id": "publisher" },
            "a": { "second": true, "first": null }
        });
        let right = json!({
            "a": { "first": null, "second": true },
            "publisher": { "id": "publisher", "publicKey": "key", "signature": "different" },
            "z": 2
        });
        let entries = vec![
            ("z.asset".to_string(), [2_u8; 32]),
            ("a.wasm".to_string(), [1_u8; 32]),
        ];
        let left = package_signature_payload(&left, &entries).unwrap();
        let right = package_signature_payload(&right, &entries).unwrap();
        assert_eq!(left, right);
        assert!(left.starts_with(PACKAGE_SIGNATURE_DOMAIN));
        assert!(!String::from_utf8_lossy(&left).contains("ignored"));
    }
}
