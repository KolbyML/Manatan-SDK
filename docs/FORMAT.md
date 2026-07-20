# `.manatan2` format and ABI

## Container

A `.manatan2` file is a ZIP archive. Required entries:

```text
manifest.json
<manifest.wasm>       WebAssembly component, not a core module
```

Optional entries are `filters.json`, `preferences.json`, the declared icon,
and manifest-declared inert assets. Paths must be relative, normalized, and
free of `.`/`..`. Declared assets may include a SHA-256 digest. Manatan caps
manifest/JSON entries at 1 MiB and components at 128 MiB. Declared icons are
capped at 8 MiB, each declared asset at 32 MiB, and all declared assets at
128 MiB. Host-mediated HTTP and processed page or media bodies are capped at
64 MiB.

## Versioning

- `schemaVersion: 2` identifies this container and manifest schema.
- `apiVersion: 2` identifies the host capability/API generation.
- WIT package `manatan:extensions@2.0.0` is the canonical ABI.
- `.manatan` is generation 1 and is never executed by the version 2 host.

Unknown additive JSON fields must be ignored. A schema or API version mismatch
is a hard install error. WIT breaking changes require a new API generation and
package suffix policy.

The Rust crate version is independent from the container/API generation.
Version 1.0 establishes the stable typed JSON baseline for `.manatan2`; later crate
releases may add fields and helpers, but must keep that wire shape backward
compatible. Any breaking WIT or JSON change requires a new API generation and
package suffix rather than silently changing generation 2.

## Manifest

Required package fields are `schemaVersion`, `id`, `name`, `version`,
`versionCode`, `apiVersion`, `wasm`, `contentType`, `publisher`, and a non-empty
`sources` array. Package, source, and publisher ids start with a lowercase ASCII
letter and contain only lowercase letters, digits, `.`, `_`, and `-`.

All sources in one package currently share the package `contentType`:
`manga`, `video`, or `novel`. This keeps install/update/repository behavior
unambiguous while allowing a source factory to ship multiple sites.
Every source must explicitly declare `contentRating` as `safe`, `suggestive`,
or `adult`; missing and `unknown` ratings are rejected so hosts can apply
store and user content controls before browsing.

`permissions.network.allow` contains URL patterns such as
`https://example.com` or `https://*.example.com`. `*` may be used only when an
extension genuinely needs unrestricted network access. `cookies`, `storage`,
`assets`, `javascript`, and `webview` default to false. Future bounded host
services are declared in `permissions.services` by exact name or wildcard
prefix. A permission grants access; it does not make a platform capability
universally available.

Host wildcards are DNS-label aware: `*.example.com` matches one non-empty
subdomain label, never the apex, a nested label, or an attacker-controlled
suffix. Explicit ports must match; a pattern without a port allows the host on
any port so self-hosted sources can use user-configured endpoints.

`assets` entries declare `path` plus optional `mimeType` and hexadecimal
`sha256`. JavaScript assets require `sha256`; APK, JAR, Dex, class, native
library, and executable assets are rejected. Only declared assets are readable
by the guest asset capability.

The archive is closed: files other than `manifest.json`, the declared
component, optional `filters.json`/`preferences.json`, the icon, and declared
assets are rejected. An executable cannot be hidden in an undeclared ZIP
member. Icons must be raster PNG, JPEG, WebP, GIF, or AVIF files.

Repository indexes must include a 64-character hexadecimal `sha256` for every
`.manatan2` artifact. The host verifies the complete download, package id, and
content type before installation; redirects or mirrors cannot substitute a
different package. This digest is required even when the repository is served
over HTTPS.

Every package is also authenticated by `publisher.id`, a 32-byte lowercase
hexadecimal Ed25519 `publisher.publicKey`, and a 64-byte lowercase hexadecimal
`publisher.signature`. The signature is calculated over:

1. the ASCII domain separator `manatan2-package-signature-v1` followed by a
   zero byte;
2. recursively key-sorted, whitespace-free JSON for `manifest.json` after
   removing only `publisher.signature`; and
3. for every non-directory ZIP entry except `manifest.json`, sorted by entry
   name: the eight-byte big-endian UTF-8 name length, the entry name, and the
   32-byte SHA-256 digest of the entry contents.

This binds the component, metadata, optional filters/preferences, icon, and all
assets to one stable publisher identity. Updates to an installed package must
use the same publisher key. Hosts may revoke package ids, complete package
digests, or publisher keys without trusting repository metadata.

## Guest operations

The Component Model interface exports explicit operations for all media types:

- common: initialization, filters, preferences, home
- manga: list/search/details/chapters/pages, URLs, lazy image resolution,
  optional image processing, related titles, alternate covers, migration
- video: list/search/details/episodes/seasons, direct streams, hosters and
  hoster streams, URLs
- novel: list/search/details/chapters/paginated chapters/text, URLs

Operation payloads are camelCase JSON strings using the strongly typed Rust
models from `manatan-sdk::model`. This keeps the Component Model function set
and host capabilities statically versioned while allowing backward-compatible
model fields to be added without exploding the WIT contract.

`guest.dispatch` is the forward-compatible path for source operations added
after the v2 baseline. The SDK exposes it on every source trait. The reserved
common conventions `auth.status`, `auth.login`, and `auth.logout` use typed
authentication models. Authentication may return a typed form, interactive
WebView, or external-browser action without blocking a Wasm call while a user
interacts; source-specific actions should use namespaced operation names and
additive JSON. Large page and video transformations have dedicated typed byte
functions so data does not expand into JSON arrays or base64.

Sources advertise supported non-baseline operation names in
`capabilities.operations`. command ports conventionally expose
`commands.describe` through dispatch and pass selected command values in the
operation request. Command categories therefore do not require fixed WIT
functions or platform-specific UI types.

## Host capabilities

- `host`: logging, clock, secure random bytes, locale, API version, bounded sleep
- `net`: typed HTTP requests/responses, timeouts, redirect policy, body bounds,
  host-coordinated keyed request intervals, and bounded parallel batches
- `store`: namespaced JSON key/value state
- `cookies`: scoped cookie snapshots and updates
- `webview`: versioned JSON browser execution/extraction
- `assets`: list/read manifest-declared inert package files
- `services`: capability discovery plus versioned JSON or typed-binary
  invocation by stable name

The baseline service name `javascript.asset.v1` evaluates only manifest-
declared package assets in an isolated ECMAScript interpreter with JSON
globals and a bounded loop count. It has no filesystem or network globals and
accepts only data-only global reads or function calls after those assets load;
it does not accept arbitrary expressions or downloaded script text. This
restriction applies to the standalone algorithm service. A permissioned WebView
may execute the scripts of the loaded page and extension-provided extraction
scripts inside that page, but it has no page-to-native interface and every
navigation and subresource remains inside the package network allowlist.
`html.select.v1` performs
bounded native CSS selection over guest-supplied HTML bytes. New service names
can be introduced without changing the v2 WIT world.
`services.invoke_binary` is reserved for services that consume or return
significant byte payloads.

The host owns transport, TLS, redirects, cookie jars, persistence, and browser
processes. It validates network permissions for request URLs, cookie lookup
URLs, redirects handled by the client, and every WebView network request,
including redirects, iframes, and subresources. Guest execution
is bounded by a memory limiter and epoch deadline. iOS uses Wasmtime's Pulley
interpreter because unsigned runtime-generated executable pages are forbidden.

WebView profiles are virtual and source-scoped. Persisted local/session storage
is keyed by serialized origin and profile id, restored before page scripts, and
captured after execution. Hosts must not expose a shared native WebView data
store as extension state or inject one origin's storage into another origin.
Native providers must not expose `addJavascriptInterface` or an equivalent
page-to-native object to extension-controlled pages. Script results are data,
not privileged commands.

Artwork fields use the typed `ImageRequest` resource. Hosts fetch these through
the extension permission boundary rather than exposing request headers or
cookies in UI-facing URLs. Redirects and `cookieUrl` are checked using the same
rules as guest HTTP requests, and `cookieUrl` must have the same origin as the
request it authenticates. Cookies are never exported to another origin.

## Media and page processing

`MangaSource::process_page_image` and `VideoSource::process_resource` receive
binary input through typed Component Model lists and return an optional binary
output with MIME type. The host imposes size, memory, and epoch limits.

Common HLS work is declarative through `VideoStream.segmentProcessing`:
cookies are selected for each resource URL from the source-scoped host jar,
while matching rules can add non-cookie headers or bounded byte transforms.

- host/URL/resource-kind matching;
- a bounded fixed leading-byte strip;
- bounded automatic TS/MP4/RIFF offset detection after JPEG/PNG/GIF disguises;
- an optional guest transform for non-standard source algorithms.

Manatan, not the component, owns HTTP listening, playlist rewriting, nested
playlist/key URLs, range forwarding, and segment streaming.

## Repository entries

Repository indexes may expose the manifest directly or a normalized entry.
Every installable entry must identify schema 2 or point to a `.manatan2`
artifact, include its media type, pin the complete artifact digest, and repeat
the package publisher identity. A normalized entry has this shape:

```json
{
  "pkgName": "manatan:com.example.my-source",
  "name": "My Source",
  "versionName": "1.0.0",
  "versionCode": 1,
  "contentType": "manga",
  "extensionType": "manatan2",
  "packageUrl": "packages/com.example.my-source.manatan2",
  "sha256": "<64 lowercase hexadecimal characters>",
  "publisher": {
    "id": "com.example",
    "publicKey": "<64 lowercase hexadecimal characters>"
  },
  "iconUrl": "icons/com.example.my-source.png"
}
```

Indexes pointing at `.manatan` are legacy metadata, not installable version 2
entries.
