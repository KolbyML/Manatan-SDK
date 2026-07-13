# Authoring `.manatan2` extensions

## Toolchain

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-tools
```

Guests target `wasm32-unknown-unknown`, not WASI. Use the SDK's host
capabilities instead of `std::fs`, sockets, subprocesses, or ambient clock and
randomness.

## Create a source crate

Add the SDK and serialization support:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
manatan-sdk = "0.3"
serde_json = "1"
```

Implement one of these traits:

- `MangaSource`: popular/latest/search, details, chapters, pages
- `VideoSource`: popular/latest/search, details, episodes, streams, optional
  seasons and hoster resolution
- `NovelSource`: popular/latest/search, details, chapters, text/html, optional
  paginated chapters

The core methods use one media-neutral `CatalogItem`. Optional methods have
safe defaults. Register sources once:

```rust,ignore
manatan_sdk::export_extension!(
    manatan_sdk::Extension::new()
        .manga("manga-a", MangaA::default())
        .manga("manga-b", MangaB::default())
);
```

Every registry id must exactly match one `sources[].id` in the manifest.

## HTTP and parsing

`client::Client` sends requests through the app's permission-checked HTTP
stack and returns bytes, text, or deserialized JSON. `html` re-exports
`scraper`'s `Html`, `Selector`, and `ElementRef`, plus helpers for text,
attributes, absolute URLs, and selectors.

```rust,ignore
let response = manatan_sdk::client::Client::browser()
    .cookies_for("https://example.com")
    .get("https://example.com/popular")
    .send()?
    .error_for_status()?;
let document = manatan_sdk::html::document(response.text()?);
```

Requests may set `timeout_ms`, `redirect_policy("follow" | "manual" |
"error")`, and `max_body_bytes`. Use `Client::send_many` for
parallel hoster extraction; batches are bounded by the host.

Use `RequestBuilder::rate_limit(key, minimum_interval_ms)` for site request
queues. The host scopes the key to the source and coordinates it across calls
and component instances; `runtime::sleep` alone only delays the current call.

Request `cookies: true` when using `cookies_for`. Raw `Cookie` headers are not
accepted; this prevents a component from bypassing cookie scoping.

For JavaScript-rendered or challenge-protected sites, request `webview: true`
and use `browser::open`/`browser::extract` with `WebViewRequest` or
`WebViewExtractRequest`. The public models cover load/DOM/network-idle waits,
selector/URL/script/event waits, preload and result scripts, HTML, cookie sync,
and bounded request/event capture. WebView access may be unavailable on
headless hosts, so sources should prefer HTTP and degrade cleanly.

Use `WebViewSession` when a challenge or login spans calls. Persistent session
state is owned by Manatan and isolated by package, source, profile id, and web
origin; it does not depend on a platform WebView's global data store. Use
`clear` for logout, and `initialStorage` only when importing known state for a
specific serialized origin. Cookies remain in the permission-checked cookie
jar and still require `cookies: true` plus `cookieUrl`.

Catalog covers, banners, chapter/episode thumbnails, alternate covers, and
novel block images use `ImageRequest`. This lets protected artwork carry a
GET or POST method, headers, optional body, and an explicit cookie scope while
the host continues to enforce network and cookie permissions. Use `ImageRequest::get`
for ordinary images; never place a raw `Cookie` header in artwork metadata.
For images embedded in `NovelText.html`, set `baseUrl` for relative paths and
set `imageContext` when the images share headers or a cookie scope. Manatan
rewrites `src`/common lazy-image attributes through the same protected artwork
path. Prefer typed image blocks when image requests differ within a chapter.

Filters are typed and keyed. `MultiSelect` values are the selected option
`value` strings (not labels or UI indexes), which keeps the same request shape
on every Manatan platform. Search receives a JSON object keyed by filter id:
selects contain an option value, multi-selects an array of values, tri-state is
`0` (ignore), `1` (include), or `2` (exclude), sort contains `index`, `value`,
and `ascending`, and groups contain a nested object keyed by child id.
Represent an include/exclude checkbox set as a `Group` of `TriState` children,
one child per option. Put reader-specific styles in `NovelText.css`; arbitrary
extension scripts are never injected into the reader UI.

For pure token/deobfuscation scripts, request `javascript: true` and use
`javascript::evaluate`. This is lighter than a WebView, shares one isolated
context across the supplied scripts/expression, and is loop-bounded. Use
`runtime::sleep` for source-required challenge delays rather than a guest
thread.

## Persistent state

`storage::{get,set,delete,list}` stores JSON values in a source-scoped host
store. Request `storage: true`. Use stable namespaces and keys; do not store
credentials unless the source genuinely requires them.

The host supplies configured source preferences on every operation. Read them
with `context::preferences()`, `context::preference_value(key)`, or the typed
`context::preference::<T>(key)` helper. `context::source_id()` identifies the
source currently handling the call.

For explicit account flows, implement `authentication_status`, `authenticate`,
and `logout`, then set the manifest `authentication` capability. Credentials
are an extensible JSON map so username/password, API-key, two-factor, and
device-code sites share the same operation convention. Do not log credentials.
If user interaction is required, return an `AuthenticationAction::Form`,
`WebView`, or `ExternalBrowser` step. A WebView step uses a source-local
persistent profile, syncs only permission-scoped cookies, may declare safe
auto-close conditions, and must remain manually closable for challenge flows.
After the step completes, the host calls `authenticate` again with collected
form values or an empty interactive request so the source can verify success.

Every source trait also has a `dispatch` hook for future or site-specific
operations such as comments and server-side bookmarks. Use a namespaced
operation name and additive request/response JSON; this path is what prevents
new optional source features from requiring a WIT revision. Advertise these
names in `sources[].capabilities.operations`. command ports should use
`commands.describe` to return their current input definitions and keep command
values in the request JSON for the operation they affect.

## Assets and binary processing

Declare scripts, fonts, lookup tables, or inner Wasm binaries in
`manifest.assets`, request `assets: true`, then read them with
`assets::{list,read}`. Asset paths are normalized and optional SHA-256 digests
are verified during installation. Small immutable data can still be embedded
with normal Rust `include_bytes!`.

Image interceptors implement `process_page_image`.
non-standard segment transforms implement `process_resource`. Both callbacks
receive typed bytes and should return `None` when no change is necessary.
Prefer declarative `SegmentProcessing` rules for fixed-prefix stripping and
media-offset detection so the host can keep the resource streaming.

Future high-cost helpers use `services::invoke` or
`services::invoke_binary`. Declare the service name in `permissions.services`
and check `services::is_available` before relying on it.

Inline content uses `NovelText.blocks`. Manga data URIs use
`PageContent::Inline`; decrypted subtitles use `MediaTrack.inline_data` or
`inline_base64`.

## Build the component

```sh
cargo build --release --target wasm32-unknown-unknown
wasm-tools component new \
  target/wasm32-unknown-unknown/release/my_source.wasm \
  -o my-source.wasm
wasm-tools validate --features component-model my-source.wasm
wasm-tools component wit my-source.wasm
```

The final inspection must export `manatan:extensions/guest@2.0.0`; any host
interfaces retained by the component must also be from
`manatan:extensions@2.0.0`. Unused imports may be removed during component
generation. A core Wasm module is not installable; Manatan validates and
instantiates the component contract before writing the package.

## Manifest and package

```json
{
  "schemaVersion": 2,
  "id": "com.example.my-source",
  "name": "My Source",
  "version": "1.0.0",
  "versionCode": 1,
  "apiVersion": 2,
  "wasm": "my-source.wasm",
  "contentType": "manga",
  "description": "Example source",
  "author": "you",
  "license": "MIT",
  "permissions": {
    "network": { "allow": ["https://example.com"] },
    "cookies": false,
    "storage": false,
    "assets": false,
    "javascript": false,
    "services": [],
    "webview": false
  },
  "sources": [{
    "id": "my-source",
    "name": "My Source",
    "lang": "en",
    "baseUrl": "https://example.com",
    "contentType": "manga",
    "contentRating": "safe",
    "capabilities": {
      "search": true,
      "latest": true,
      "filters": false,
      "preferences": false,
      "home": false,
      "urlResolution": true
    }
  }]
}
```

An asset declaration is separate from the icon:

```json
"assets": [{
  "path": "assets/deobfuscator.js",
  "mimeType": "text/javascript",
  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
}]
```

A proxy stream with prefixed or disguised media segments can express its
processing behavior without opening a local server:

```rust,ignore
stream.requires_proxy = true;
stream.segment_processing = Some(SegmentProcessing {
    rewrite_playlist: true,
    rules: vec![
        SegmentRule {
            host_patterns: vec!["*.media.example".into(), "cdn.example".into()],
            resource_types: vec![MediaResourceKind::Segment],
            strip_prefix_bytes: Some(252),
            ..Default::default()
        },
        SegmentRule {
            resource_types: vec![MediaResourceKind::Segment],
            auto_detect_media_offset: true,
            probe_bytes: Some(4096),
            ..Default::default()
        },
    ],
    ..Default::default()
});
```

Package the component plus every declared asset:

```sh
zip my-source.manatan2 manifest.json my-source.wasm assets/deobfuscator.js icon.png
```

Do not rename a legacy package. `.manatan` used a different core-module ABI;
the source must be rebuilt against this SDK and componentized.
