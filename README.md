# Manatan SDK

The public SDK and canonical WIT contract for Manatan manga, video, and novel
extensions. Extensions compile to sandboxed WebAssembly **components**, run by
Wasmtime in Manatan, and are packaged as `.manatan2` archives.

This repository contains only the SDK, contract, and documentation. It does
not contain extension implementations or depend on Manatan's private host.

## Design

- A single component may register one or more sources with the matching
  `Extension::manga`, `Extension::video`, or `Extension::novel` builder. Every
  source in a package uses the package's declared media kind.
- HTTP (including bounded parallel batches), cookies, storage, declared
  assets, clock/randomness, bounded JavaScript, and WebView execution are host
  capabilities. Guests have no ambient filesystem, socket, or process access.
- Manifests declare network origins and privileged capabilities. The host
  enforces them on every call.
- Protected artwork uses typed requests while UI-facing URLs remain opaque;
  browser profiles persist origin-isolated challenge/login state.
- Filters and authentication interactions have media-neutral typed models,
  including multi-select filters and form/WebView/external-browser steps.
- ABI version 2 uses the WebAssembly Component Model. Legacy `.manatan` core
  modules are intentionally incompatible and obsolete.

## Minimal shape

```rust,ignore
use manatan_sdk::{CatalogItem, MangaChapter, MangaPage, MangaSource, Paged, Result};

struct MySource;

impl MangaSource for MySource {
    fn popular(&mut self, page: u32) -> Result<Paged<CatalogItem>> { /* ... */ }
    fn search(&mut self, query: &str, page: u32, filters: &serde_json::Value)
        -> Result<Paged<CatalogItem>> { /* ... */ }
    fn details(&mut self, item: CatalogItem) -> Result<CatalogItem> { /* ... */ }
    fn chapters(&mut self, item: CatalogItem) -> Result<Vec<MangaChapter>> { /* ... */ }
    fn pages(&mut self, item: CatalogItem, chapter: MangaChapter)
        -> Result<Vec<MangaPage>> { /* ... */ }
}

manatan_sdk::export_extension!(
    manatan_sdk::Extension::new().manga("my-source", MySource)
);
```

Build it for `wasm32-unknown-unknown`, componentize it with `wasm-tools`, and
zip the component with `manifest.json` as `name.manatan2`.

See [Authoring](docs/AUTHORING.md) and the [format specification](docs/FORMAT.md).

## Repository layout

```text
crates/manatan-sdk/                 Rust traits, models, helpers, and bindings
crates/manatan-sdk/wit/world.wit   canonical Component Model contract
docs/AUTHORING.md                   build and authoring guide
docs/FORMAT.md                      schema, ABI, security, and repository format
```

Licensed under MIT.
