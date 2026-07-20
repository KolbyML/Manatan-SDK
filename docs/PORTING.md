# Porting existing source integrations

`.manatan2` is a source-code porting target, not a wrapper around APK, JAR, or
downloaded plugin programs. A port rewrites the source logic in Rust and
compiles it into a WebAssembly component. The resulting package can preserve
the source's observable catalog, search, details, chapter, page, episode,
stream, and novel behavior without loading upstream executable artifacts.

This is the only supported generation. There is no legacy ABI, APK/JAR
fallback, raw plugin loader, or native class bridge.

## Compatibility mapping

| Upstream facility | `.manatan2` equivalent |
| --- | --- |
| OkHttp request builders and interceptors | `client::Client`, request headers/body/redirect policy, keyed rate limits, scoped cookies, and source-local Rust wrapper functions |
| OkHttp multipart bodies | `RequestBuilder::multipart` and `MultipartPart` |
| Kotlin coroutines / JavaScript promises | synchronous guest logic plus bounded `Client::send_many`; the host performs parallel I/O |
| Jsoup / Cheerio / htmlparser2 | `html` helpers or the bounded `html.select.v1` host service |
| SharedPreferences / MMKV / AsyncStorage | source-scoped `storage` and typed source preferences |
| Android `Context`, activities, dialogs, and toasts | typed filters, preferences, authentication actions, URLs, and operation results presented by Manatan |
| WebView challenge/login | source-scoped `WebViewSession`, cookie synchronization, storage snapshots, waits, scripts, and request capture |
| QuickJS token/deobfuscation code | Rust implementation, or verified package assets through `javascript.asset.v1` |
| Image interceptors / scrambled pages | `ImageRequest`, `resolve_page_image`, and bounded `process_page_image` bytes |
| NanoHTTPD media servers | host-owned playlist/resource proxy, per-resource header rules, scoped cookies, declarative segment rules, and bounded `process_resource` bytes |
| Video extractors and hosters | typed hosters, streams, HLS/DASH flags, tracks, DRM metadata, torrent/debrid descriptors, and headers |
| Novel chapter HTML and CSS | `NovelText`, typed content blocks, base URL, image context, and chapter pagination |
| Extension commands and site-specific features | namespaced `dispatch` operations advertised in source capabilities |
| Bundled lookup tables, fonts, scripts, or inner Wasm data | manifest-declared, optionally SHA-256-pinned assets |

The guest is ordinary Rust and remains Turing-complete inside its bounded
WebAssembly sandbox. Parsing, hashing, codecs, cryptography, protobuf, custom
date logic, and site-specific algorithms should use Rust crates that support
`wasm32-unknown-unknown`. Those libraries become part of the reviewed
`.manatan2` component; they are not downloaded at runtime.

## Browser boundary

Browser assistance has three deliberately separate layers:

1. The extension asks Manatan for a typed browser operation.
2. Manatan drives the platform WebView and evaluates the requested script.
3. The page returns a JSON-compatible value through a data-only result
   envelope.

There is no page-to-Java/Kotlin/Swift object. A page cannot ask native code to
fetch a URL, open a socket, read a file, install code, or invoke an extension
operation. When a legacy extractor used a JavaScript/native fetch bridge, port
the orchestration to Rust: make permission-checked HTTP calls with `Client`,
then run packaged pure JavaScript only if the algorithm cannot reasonably be
rewritten. Remote page JavaScript may execute as part of normal WebView
navigation, within the manifest network allowlist.

## Media-server replacement

A source must not start an embedded HTTP server in a `.manatan2` guest. Return the
real upstream stream and set `requires_proxy`. Manatan rewrites HLS playlists
through its own loopback server. `SegmentProcessing` can:

- attach only cookies applicable to each resource URL with `cookies: true`;
- merge headers only for matching resource kinds, URL fragments, or hosts;
- rewrite nested playlists through the host proxy;
- strip fixed prefixes or auto-detect disguised media offsets; and
- call `process_resource` for a bounded source-specific byte transform.

The player never receives upstream cookies or private headers directly. The
host checks every playlist, segment, key, subtitle, audio, video, and redirect
URL against the package's network permissions.

## Required rewrites

Some upstream mechanisms are implementation details that must not survive the
port even when the end-user feature can be preserved:

- APK/JAR/Dex loading becomes compiled Rust in the `.manatan2` component.
- Downloaded plugin text and dynamic `Function(...)` loaders become
  compiled Rust; the original TypeScript is reference material only.
- Downloaded token scripts become Rust or pinned package assets. The SDK has
  no arbitrary-script or source-expression evaluator; packaged functions are
  invoked with JSON data.
- `addJavascriptInterface`, WK script-message commands, and similar privileged
  page bridges are not available.
- Raw sockets, local servers, subprocesses, ambient filesystem paths, and
  native reflection are replaced by typed host capabilities.
- Trust-all TLS, disabled hostname verification, or certificate bypasses are
  not reproducible. The upstream site must work with platform TLS.
- Code or content that violates store policy does not become acceptable by
  moving it into an extension.

These exclusions remove unsafe mechanisms, not source categories. A source
whose catalog behavior used one of them can still be ported when the behavior
is implemented through the safe equivalent. A source whose server itself
requires invalid TLS or whose only behavior is a policy violation cannot be
made store-compliant by an SDK feature.

The format can represent adult-rated sources for non-Play distributions, but
the Google Play build does not list, install, or load any package containing an
`adult` source. Keep adult and non-adult sources in separate packages so the
non-adult package remains eligible for Play distribution.

## Port acceptance checklist

A port is complete when:

1. Every network origin used by HTTP, browser subresources, images, streams,
   redirects, and cookie scopes is declared in `permissions.network.allow`.
2. The package contains only the Component Model guest and declared inert
   assets; no APK, JAR, Dex, native library, or raw plugin program is loaded.
   Its repository entry includes the package SHA-256 and matching package id
   and content type.
3. Browser scripts return data only and do not depend on a native page bridge.
   Dynamic page JavaScript stays inside the isolated WebView; it cannot invoke
   Android, iOS, filesystem, socket, installation, or extension host commands.
4. Token scripts are Rust or declared package assets.
5. Cookies use host scoping rather than raw cookie metadata.
   A cookie lookup URL has the same origin as its request, and media cookies
   are selected separately for each proxied resource URL.
6. Media processing uses the host proxy rather than a guest-owned server.
7. Binary processing has explicit byte limits and returns `None` when no
   transform is necessary.
8. Filters, preferences, authentication, content rating, and optional
   operations are accurately declared. Every source uses an explicit `safe`,
   `suggestive`, or `adult` rating; `unknown` packages are rejected.
9. The package is signed by a stable Ed25519 publisher key and repository
   metadata identifies that same publisher key.
10. The component passes `wasm-tools validate` and the host conformance tests.
11. The source is tested against representative catalog, search, details,
    chapter/episode, page/text, login/challenge, and media cases.
