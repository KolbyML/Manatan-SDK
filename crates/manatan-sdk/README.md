# Manatan SDK

`manatan-sdk` is the public Rust SDK for building Manatan manga, video, and
novel extensions as sandboxed WebAssembly components packaged in `.manatan2`
archives.

It provides:

- source traits and media-neutral models;
- Component Model export bindings;
- host-mediated HTTP, cookies, storage, assets, JavaScript, and browser access;
- typed manga page and video resource processing callbacks; and
- manifest, authentication, preference, filter, and host-service models.

## Dependency

```toml
[dependencies]
manatan-sdk = "0.3"
```

Build extension guests for `wasm32-unknown-unknown` and componentize the output
against the WIT world distributed with this crate.

The complete authoring guide and `.manatan2` format specification are in the
[Manatan SDK repository](https://github.com/KolbyML/Manatan-SDK).

Licensed under MIT.
