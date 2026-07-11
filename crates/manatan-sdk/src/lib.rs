//! Public SDK for `.manatan2` manga, video, and novel extensions.
//!
//! The SDK is independent of Manatan's private host. It exposes source traits, host-mediated HTTP and browser
//! capabilities, and the Component Model export glue used by Wasmtime.

mod source;

pub mod assets;
pub mod browser;
pub mod client;
pub mod context;
pub mod html;
pub mod javascript;
pub mod manifest;
pub mod model;
pub mod runtime;
pub mod services;
pub mod storage;

pub use model::*;
pub use source::{Error, Extension, MangaSource, NovelSource, Result, VideoSource};

wit_bindgen::generate!({
    world: "extension",
    path: "wit",
    pub_export_macro: true,
    export_macro_name: "export_extension_raw",
});

pub use crate::manatan::extensions::{
    assets as asset_host, cookies, host, net, services as service_host, store, webview,
};

#[doc(hidden)]
pub use crate::exports::manatan::extensions::guest::Guest as __Guest;

#[doc(hidden)]
pub use crate::exports::manatan::extensions::guest::BinaryOutput as __BinaryOutput;

#[doc(hidden)]
pub fn __extension_call(
    extension: &mut Extension,
    source_id: String,
    operation: &str,
    request_json: String,
) -> std::result::Result<String, String> {
    extension
        .call(&source_id, operation, &request_json)
        .map_err(|error| error.message)
}

#[doc(hidden)]
#[macro_export]
macro_rules! __guest_method {
    ($name:ident, $operation:literal) => {
        fn $name(
            source_id: ::std::string::String,
            request_json: ::std::string::String,
        ) -> ::core::result::Result<::std::string::String, ::std::string::String> {
            Self::__with(|extension| {
                $crate::__extension_call(extension, source_id, $operation, request_json)
            })
        }
    };
}

/// Export a registry of one or more media sources as a WebAssembly component.
///
/// The expression is evaluated once per component instance:
///
/// ```ignore
/// manatan_sdk::export_extension!(
///     manatan_sdk::Extension::new()
///         .manga("manga-source", MangaSourceImpl::default())
///         .video("video-source", VideoSourceImpl::default())
/// );
/// ```
#[macro_export]
macro_rules! export_extension {
    ($registry:expr $(,)?) => {
        #[doc(hidden)]
        struct __ManatanMediaExtensionComponent;

        impl __ManatanMediaExtensionComponent {
            fn __with<R>(
                callback: impl FnOnce(&mut $crate::Extension) -> R,
            ) -> R {
                ::std::thread_local! {
                    static STATE: ::std::cell::RefCell<::core::option::Option<$crate::Extension>> =
                        const { ::std::cell::RefCell::new(::core::option::Option::None) };
                }
                STATE.with(|cell| {
                    let mut slot = cell.borrow_mut();
                    let extension = slot.get_or_insert_with(|| $registry);
                    callback(extension)
                })
            }
        }

        impl $crate::__Guest for __ManatanMediaExtensionComponent {
            fn init() -> ::core::result::Result<(), ::std::string::String> {
                Self::__with(|extension| extension.init().map_err(|error| error.message))
            }

            fn dispatch(
                source_id: ::std::string::String,
                operation: ::std::string::String,
                request_json: ::std::string::String,
            ) -> ::core::result::Result<::std::string::String, ::std::string::String> {
                Self::__with(|extension| {
                    $crate::__extension_call(extension, source_id, &operation, request_json)
                })
            }

            $crate::__guest_method!(filters, "filters");
            $crate::__guest_method!(preferences, "preferences");
            $crate::__guest_method!(home, "home");

            $crate::__guest_method!(manga_list, "manga.list");
            $crate::__guest_method!(manga_search, "manga.search");
            $crate::__guest_method!(manga_details, "manga.details");
            $crate::__guest_method!(manga_chapters, "manga.chapters");
            $crate::__guest_method!(manga_pages, "manga.pages");
            $crate::__guest_method!(manga_home, "manga.home");
            $crate::__guest_method!(manga_item_url, "manga.item-url");
            $crate::__guest_method!(manga_chapter_url, "manga.chapter-url");
            $crate::__guest_method!(manga_handle_url, "manga.handle-url");
            $crate::__guest_method!(manga_prepare_chapter, "manga.prepare-chapter");
            $crate::__guest_method!(manga_resolve_page_image, "manga.resolve-page-image");
            $crate::__guest_method!(manga_process_page_image, "manga.process-page-image");

            fn manga_process_page_bytes(
                source_id: ::std::string::String,
                context_json: ::std::string::String,
                bytes: ::std::vec::Vec<u8>,
                mime_type: ::core::option::Option<::std::string::String>,
            ) -> ::core::result::Result<
                ::core::option::Option<$crate::__BinaryOutput>,
                ::std::string::String,
            > {
                Self::__with(|extension| {
                    extension
                        .process_page_bytes(
                            &source_id,
                            &context_json,
                            &bytes,
                            mime_type.as_deref(),
                        )
                        .map(|output| output.map(|output| {
                            $crate::__BinaryOutput {
                                bytes: output.bytes,
                                mime_type: output.mime_type,
                            }
                        }))
                        .map_err(|error| error.message)
                })
            }
            $crate::__guest_method!(manga_alternate_covers, "manga.alternate-covers");
            $crate::__guest_method!(manga_related, "manga.related");
            $crate::__guest_method!(manga_migrate, "manga.migrate");

            $crate::__guest_method!(video_list, "video.list");
            $crate::__guest_method!(video_search, "video.search");
            $crate::__guest_method!(video_details, "video.details");
            $crate::__guest_method!(video_episodes, "video.episodes");
            $crate::__guest_method!(video_seasons, "video.seasons");
            $crate::__guest_method!(video_hosters, "video.hosters");
            $crate::__guest_method!(video_streams, "video.streams");
            $crate::__guest_method!(video_hoster_streams, "video.hoster-streams");
            $crate::__guest_method!(video_home, "video.home");
            $crate::__guest_method!(video_item_url, "video.item-url");
            $crate::__guest_method!(video_episode_url, "video.episode-url");
            $crate::__guest_method!(video_handle_url, "video.handle-url");

            fn video_process_resource(
                source_id: ::std::string::String,
                context_json: ::std::string::String,
                bytes: ::std::vec::Vec<u8>,
                mime_type: ::core::option::Option<::std::string::String>,
            ) -> ::core::result::Result<
                ::core::option::Option<$crate::__BinaryOutput>,
                ::std::string::String,
            > {
                Self::__with(|extension| {
                    extension
                        .process_video_resource(
                            &source_id,
                            &context_json,
                            &bytes,
                            mime_type.as_deref(),
                        )
                        .map(|output| output.map(|output| {
                            $crate::__BinaryOutput {
                                bytes: output.bytes,
                                mime_type: output.mime_type,
                            }
                        }))
                        .map_err(|error| error.message)
                })
            }

            $crate::__guest_method!(novel_list, "novel.list");
            $crate::__guest_method!(novel_search, "novel.search");
            $crate::__guest_method!(novel_details, "novel.details");
            $crate::__guest_method!(novel_chapters, "novel.chapters");
            $crate::__guest_method!(novel_chapters_page, "novel.chapters-page");
            $crate::__guest_method!(novel_text, "novel.text");
            $crate::__guest_method!(novel_home, "novel.home");
            $crate::__guest_method!(novel_item_url, "novel.item-url");
            $crate::__guest_method!(novel_chapter_url, "novel.chapter-url");
            $crate::__guest_method!(novel_handle_url, "novel.handle-url");
        }

        $crate::export_extension_raw!(__ManatanMediaExtensionComponent with_types_in $crate);
    };
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    struct TestManga;

    impl MangaSource for TestManga {
        fn popular(&mut self, _page: u32) -> Result<Paged<CatalogItem>> {
            let title =
                context::preference::<String>("title")?.unwrap_or_else(|| "One".to_string());
            Ok(Paged::new(vec![CatalogItem::new("/one", title)], false))
        }
        fn search(
            &mut self,
            _query: &str,
            _page: u32,
            _filters: &serde_json::Value,
        ) -> Result<Paged<CatalogItem>> {
            Ok(Paged::default())
        }
        fn details(&mut self, item: CatalogItem) -> Result<CatalogItem> {
            Ok(item)
        }
        fn chapters(&mut self, _item: CatalogItem) -> Result<Vec<MangaChapter>> {
            Ok(Vec::new())
        }
        fn pages(&mut self, _item: CatalogItem, _chapter: MangaChapter) -> Result<Vec<MangaPage>> {
            Ok(Vec::new())
        }

        fn authentication_status(&mut self) -> Result<AuthenticationState> {
            Ok(AuthenticationState {
                authenticated: true,
                account_name: Some("test-user".to_string()),
                ..AuthenticationState::default()
            })
        }

        fn dispatch(
            &mut self,
            operation: &str,
            request: &serde_json::Value,
        ) -> Result<Option<serde_json::Value>> {
            Ok((operation == "comments.list").then(|| request.clone()))
        }
    }

    #[test]
    fn registry_dispatches_typed_sources() {
        let mut extension = Extension::new().manga("test", TestManga);
        let result = extension
            .call(
                "test",
                "manga.list",
                &json!({
                    "listing": "popular",
                    "page": 1,
                    "preferences": { "title": "Configured" }
                })
                .to_string(),
            )
            .unwrap();
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(result["entries"][0]["title"], "Configured");

        let auth: serde_json::Value =
            serde_json::from_str(&extension.call("test", "auth.status", "{}").unwrap()).unwrap();
        assert_eq!(auth["accountName"], "test-user");

        let custom: serde_json::Value = serde_json::from_str(
            &extension
                .call("test", "comments.list", r#"{"page":2}"#)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(custom["page"], 2);
        assert_eq!(context::source_id(), None);
    }

    #[test]
    fn page_request_and_processed_bytes_use_host_wire_shapes() {
        let request = serde_json::to_value(PageContent::Request(ImageRequest {
            url: "https://example.test/page.jpg".to_string(),
            ..ImageRequest::default()
        }))
        .unwrap();
        assert_eq!(
            request
                .pointer("/request/url")
                .and_then(serde_json::Value::as_str),
            Some("https://example.test/page.jpg")
        );

        let processed = serde_json::to_value(ProcessedImage {
            bytes: vec![1, 2, 3],
            mime_type: Some("image/png".to_string()),
        })
        .unwrap();
        assert_eq!(processed["bytes"], json!([1, 2, 3]));
        assert_eq!(processed["mimeType"], "image/png");
    }

    #[test]
    fn ecosystem_models_keep_additive_wire_shapes() {
        let stream = VideoStream {
            url: "https://cdn.example.test/master.m3u8".to_string(),
            requires_proxy: true,
            segment_processing: Some(SegmentProcessing {
                rewrite_playlist: true,
                rules: vec![SegmentRule {
                    host_patterns: vec!["*.example.test".to_string()],
                    strip_prefix_bytes: Some(252),
                    auto_detect_media_offset: true,
                    ..SegmentRule::default()
                }],
                ..SegmentProcessing::default()
            }),
            ..VideoStream::default()
        };
        let stream = serde_json::to_value(stream).unwrap();
        assert_eq!(
            stream["segmentProcessing"]["rules"][0]["stripPrefixBytes"],
            252
        );

        let hoster = VideoHoster {
            key: "server-1".to_string(),
            name: "Server 1".to_string(),
            internal_data: Some("opaque-token".to_string()),
            resolved_streams: vec![VideoStream {
                url: "https://cdn.example.test/video.mp4".to_string(),
                ..VideoStream::default()
            }],
            ..VideoHoster::default()
        };
        let hoster = serde_json::to_value(hoster).unwrap();
        assert_eq!(hoster["internalData"], "opaque-token");
        assert_eq!(
            hoster["resolvedStreams"][0]["url"],
            "https://cdn.example.test/video.mp4"
        );

        let text = NovelText {
            blocks: vec![
                NovelContentBlock::Text {
                    text: "Paragraph".to_string(),
                    html: false,
                },
                NovelContentBlock::Image {
                    url: "https://example.test/illustration.jpg".to_string(),
                    headers: Headers::default(),
                    alt: Some("Illustration".to_string()),
                },
            ],
            ..NovelText::default()
        };
        let text = serde_json::to_value(text).unwrap();
        assert_eq!(text["blocks"][0]["type"], "text");
        assert_eq!(text["blocks"][1]["type"], "image");

        let preference = PreferenceDefinition::MultiSelect {
            key: "servers".to_string(),
            title: "Servers".to_string(),
            summary: None,
            options: vec![OptionItem {
                label: "One".to_string(),
                value: "one".to_string(),
            }],
            default: vec!["one".to_string()],
        };
        assert_eq!(
            serde_json::to_value(preference).unwrap()["type"],
            "multiSelect"
        );

        let browser_request = browser::WebViewRequest {
            url: "https://example.test/challenge".to_string(),
            wait_for: Some(browser::WebViewWait::Selector {
                selector: "#loaded".to_string(),
            }),
            return_html: true,
            ..browser::WebViewRequest::default()
        };
        let browser_request = serde_json::to_value(browser_request).unwrap();
        assert_eq!(browser_request["waitFor"]["type"], "selector");
    }
}
