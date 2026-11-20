# Manatan SDK

`manatan-sdk` is the public Rust SDK for building Manatan manga, video, and
novel extensions as sandboxed WebAssembly components packaged in `.manatan2`
archives.

It provides:

- source traits and media-neutral models;
- Component Model export bindings;
- host-mediated HTTP, cookies, storage, assets, JavaScript, browser access, and
  native batched HTML selection;
- bounded HTTP-first browser challenge recovery with source-isolated cookies;
- typed manga page, video processing, and host-materialized novel resources;
- newest-first sequential media lists with host-owned source positions; and
- manifest, authentication, preference, filter, and host-service models.

## Dependency

```toml
[dependencies]
manatan-sdk = "2.1"
```

SDK 2.0 removes extension-authored `source_order` from manga and novel chapter
models. Manga chapters, video episodes, and novel chapters must be returned
newest-first; Manatan derives its persisted source positions from list order.

Build extension guests for `wasm32-unknown-unknown` and componentize the output
against the WIT world distributed with this crate.

The complete authoring guide and `.manatan2` format specification are in the
[Manatan SDK repository](https://github.com/KolbyML/Manatan-SDK).

Licensed under MIT.
