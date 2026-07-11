use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::model::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Manga source contract, modeled on the modern flow.
pub trait MangaSource: 'static {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn popular(&mut self, page: u32) -> Result<Paged<CatalogItem>>;

    fn latest(&mut self, _page: u32) -> Result<Paged<CatalogItem>> {
        Ok(Paged::default())
    }

    fn listing(
        &mut self,
        listing: &str,
        page: u32,
        _filters: &Value,
    ) -> Result<Paged<CatalogItem>> {
        match listing {
            "popular" => self.popular(page),
            "latest" => self.latest(page),
            other => Err(Error::new(format!("unknown manga listing {other:?}"))),
        }
    }

    fn search(&mut self, query: &str, page: u32, filters: &Value) -> Result<Paged<CatalogItem>>;
    fn details(&mut self, item: CatalogItem) -> Result<CatalogItem>;
    fn chapters(&mut self, item: CatalogItem) -> Result<Vec<MangaChapter>>;
    fn pages(&mut self, item: CatalogItem, chapter: MangaChapter) -> Result<Vec<MangaPage>>;

    fn filters(&mut self) -> Result<Vec<FilterDefinition>> {
        Ok(Vec::new())
    }

    fn preferences(&mut self) -> Result<Vec<PreferenceDefinition>> {
        Ok(Vec::new())
    }

    fn authentication_status(&mut self) -> Result<AuthenticationState> {
        Ok(AuthenticationState::default())
    }

    fn authenticate(&mut self, _request: AuthenticationRequest) -> Result<AuthenticationState> {
        Err(Error::new("this source does not support authentication"))
    }

    fn logout(&mut self) -> Result<AuthenticationState> {
        Ok(AuthenticationState::default())
    }

    /// Handle source-specific or future operations delivered through the
    /// stable `guest.dispatch` export. Return `None` when unsupported.
    fn dispatch(&mut self, _operation: &str, _request: &Value) -> Result<Option<Value>> {
        Ok(None)
    }

    fn home(&mut self) -> Result<Vec<HomeSection>> {
        Ok(Vec::new())
    }

    fn item_url(&mut self, item: &CatalogItem) -> Result<Option<String>> {
        Ok(item.url.clone().or_else(|| Some(item.key.clone())))
    }

    fn chapter_url(
        &mut self,
        _item: &CatalogItem,
        chapter: &MangaChapter,
    ) -> Result<Option<String>> {
        Ok(chapter.url.clone().or_else(|| Some(chapter.key.clone())))
    }

    fn handle_url(&mut self, _url: &str) -> Result<Option<UrlResolveResult>> {
        Ok(None)
    }

    fn prepare_chapter(
        &mut self,
        _item: &CatalogItem,
        chapter: MangaChapter,
    ) -> Result<MangaChapter> {
        Ok(chapter)
    }

    fn resolve_page_image(
        &mut self,
        _item: &CatalogItem,
        _chapter: &MangaChapter,
        page: &MangaPage,
    ) -> Result<Option<MangaPageImage>> {
        let image = match &page.content {
            PageContent::Url { url, context } => Some(MangaPageImage {
                url: url.clone(),
                headers: context.clone().unwrap_or_else(|| page.headers.clone()),
            }),
            PageContent::Request(request) => Some(MangaPageImage {
                url: request.url.clone(),
                headers: request.headers.clone(),
            }),
            PageContent::Lazy {
                url: Some(url),
                context,
                ..
            } => Some(MangaPageImage {
                url: url.clone(),
                headers: context.clone().unwrap_or_else(|| page.headers.clone()),
            }),
            _ => None,
        };
        Ok(image)
    }

    fn process_page_image(
        &mut self,
        _item: &CatalogItem,
        _chapter: &MangaChapter,
        _page: &MangaPage,
        _image: &[u8],
        _mime_type: Option<&str>,
    ) -> Result<Option<ProcessedImage>> {
        Ok(None)
    }

    fn alternate_covers(&mut self, _item: &CatalogItem) -> Result<Vec<AlternateCover>> {
        Ok(Vec::new())
    }

    fn related(&mut self, _item: &CatalogItem) -> Result<Vec<CatalogItem>> {
        Ok(Vec::new())
    }

    fn migrate(
        &mut self,
        _item: &CatalogItem,
        _target_source_id: &str,
    ) -> Result<Vec<CatalogItem>> {
        Ok(Vec::new())
    }
}

/// Video source contract, including season, hoster, and stream
/// resolution without forcing a source to use hosters.
pub trait VideoSource: 'static {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn popular(&mut self, page: u32) -> Result<Paged<CatalogItem>>;
    fn latest(&mut self, _page: u32) -> Result<Paged<CatalogItem>> {
        Ok(Paged::default())
    }
    fn listing(
        &mut self,
        listing: &str,
        page: u32,
        _filters: &Value,
    ) -> Result<Paged<CatalogItem>> {
        match listing {
            "popular" => self.popular(page),
            "latest" => self.latest(page),
            other => Err(Error::new(format!("unknown video listing {other:?}"))),
        }
    }
    fn search(&mut self, query: &str, page: u32, filters: &Value) -> Result<Paged<CatalogItem>>;
    fn details(&mut self, item: CatalogItem) -> Result<CatalogItem>;
    fn episodes(&mut self, item: CatalogItem) -> Result<Vec<VideoEpisode>>;
    fn streams(&mut self, item: CatalogItem, episode: VideoEpisode) -> Result<Vec<VideoStream>>;
    fn seasons(&mut self, _item: CatalogItem) -> Result<Vec<CatalogItem>> {
        Ok(Vec::new())
    }
    fn hosters(&mut self, _item: CatalogItem, _episode: VideoEpisode) -> Result<Vec<VideoHoster>> {
        Ok(Vec::new())
    }
    fn hoster_streams(
        &mut self,
        item: CatalogItem,
        episode: VideoEpisode,
        _hoster: VideoHoster,
    ) -> Result<Vec<VideoStream>> {
        self.streams(item, episode)
    }
    fn filters(&mut self) -> Result<Vec<FilterDefinition>> {
        Ok(Vec::new())
    }
    fn preferences(&mut self) -> Result<Vec<PreferenceDefinition>> {
        Ok(Vec::new())
    }

    fn authentication_status(&mut self) -> Result<AuthenticationState> {
        Ok(AuthenticationState::default())
    }

    fn authenticate(&mut self, _request: AuthenticationRequest) -> Result<AuthenticationState> {
        Err(Error::new("this source does not support authentication"))
    }

    fn logout(&mut self) -> Result<AuthenticationState> {
        Ok(AuthenticationState::default())
    }

    /// Handle source-specific or future operations delivered through the
    /// stable `guest.dispatch` export. Return `None` when unsupported.
    fn dispatch(&mut self, _operation: &str, _request: &Value) -> Result<Option<Value>> {
        Ok(None)
    }
    fn home(&mut self) -> Result<Vec<HomeSection>> {
        Ok(Vec::new())
    }
    fn item_url(&mut self, item: &CatalogItem) -> Result<Option<String>> {
        Ok(item.url.clone().or_else(|| Some(item.key.clone())))
    }
    fn episode_url(
        &mut self,
        _item: &CatalogItem,
        episode: &VideoEpisode,
    ) -> Result<Option<String>> {
        Ok(episode.url.clone().or_else(|| Some(episode.key.clone())))
    }
    fn handle_url(&mut self, _url: &str) -> Result<Option<UrlResolveResult>> {
        Ok(None)
    }

    /// Optionally transform a proxied playlist segment, key, subtitle, or
    /// other media resource. Common prefix stripping should use
    /// `VideoStream::segment_processing`; this callback is for source-specific
    /// transforms that cannot be expressed declaratively.
    fn process_resource(
        &mut self,
        _context: &Value,
        _bytes: &[u8],
        _mime_type: Option<&str>,
    ) -> Result<Option<ProcessedMedia>> {
        Ok(None)
    }
}

/// Novel source contract, combining text-first API with paginated
/// chapter support.
pub trait NovelSource: 'static {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn popular(&mut self, page: u32) -> Result<Paged<CatalogItem>>;
    fn latest(&mut self, _page: u32) -> Result<Paged<CatalogItem>> {
        Ok(Paged::default())
    }
    fn listing(
        &mut self,
        listing: &str,
        page: u32,
        _filters: &Value,
    ) -> Result<Paged<CatalogItem>> {
        match listing {
            "popular" => self.popular(page),
            "latest" => self.latest(page),
            other => Err(Error::new(format!("unknown novel listing {other:?}"))),
        }
    }
    fn search(&mut self, query: &str, page: u32, filters: &Value) -> Result<Paged<CatalogItem>>;
    fn details(&mut self, item: CatalogItem) -> Result<CatalogItem>;
    fn chapters(&mut self, item: CatalogItem) -> Result<Vec<NovelChapter>>;
    fn text(&mut self, item: CatalogItem, chapter: NovelChapter) -> Result<NovelText>;
    fn chapters_page(&mut self, item: CatalogItem, page: u32) -> Result<NovelChapterPage> {
        Ok(NovelChapterPage {
            entries: self.chapters(item)?,
            has_next_page: false,
            page_count: Some(page.max(1)),
        })
    }
    fn filters(&mut self) -> Result<Vec<FilterDefinition>> {
        Ok(Vec::new())
    }
    fn preferences(&mut self) -> Result<Vec<PreferenceDefinition>> {
        Ok(Vec::new())
    }

    fn authentication_status(&mut self) -> Result<AuthenticationState> {
        Ok(AuthenticationState::default())
    }

    fn authenticate(&mut self, _request: AuthenticationRequest) -> Result<AuthenticationState> {
        Err(Error::new("this source does not support authentication"))
    }

    fn logout(&mut self) -> Result<AuthenticationState> {
        Ok(AuthenticationState::default())
    }

    /// Handle source-specific or future operations delivered through the
    /// stable `guest.dispatch` export. Return `None` when unsupported.
    fn dispatch(&mut self, _operation: &str, _request: &Value) -> Result<Option<Value>> {
        Ok(None)
    }
    fn home(&mut self) -> Result<Vec<HomeSection>> {
        Ok(Vec::new())
    }
    fn item_url(&mut self, item: &CatalogItem) -> Result<Option<String>> {
        Ok(item.url.clone().or_else(|| Some(item.key.clone())))
    }
    fn chapter_url(
        &mut self,
        _item: &CatalogItem,
        chapter: &NovelChapter,
    ) -> Result<Option<String>> {
        Ok(chapter.url.clone().or_else(|| Some(chapter.key.clone())))
    }
    fn handle_url(&mut self, _url: &str) -> Result<Option<UrlResolveResult>> {
        Ok(None)
    }
}

trait ErasedSource {
    fn id(&self) -> &str;
    fn init(&mut self) -> Result<()>;
    fn call(&mut self, operation: &str, request: Value) -> Result<Value>;
    fn process_page_bytes(
        &mut self,
        _context: &Value,
        _bytes: &[u8],
        _mime_type: Option<&str>,
    ) -> Result<Option<ProcessedImage>> {
        Err(Error::new("source does not support manga page processing"))
    }
    fn process_video_resource(
        &mut self,
        _context: &Value,
        _bytes: &[u8],
        _mime_type: Option<&str>,
    ) -> Result<Option<ProcessedMedia>> {
        Err(Error::new(
            "source does not support video resource processing",
        ))
    }
}

pub struct Extension {
    sources: Vec<Box<dyn ErasedSource>>,
}

impl Default for Extension {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn manga<T: MangaSource>(mut self, id: impl Into<String>, source: T) -> Self {
        self.sources.push(Box::new(MangaEntry {
            id: id.into(),
            source,
        }));
        self
    }

    pub fn video<T: VideoSource>(mut self, id: impl Into<String>, source: T) -> Self {
        self.sources.push(Box::new(VideoEntry {
            id: id.into(),
            source,
        }));
        self
    }

    pub fn novel<T: NovelSource>(mut self, id: impl Into<String>, source: T) -> Self {
        self.sources.push(Box::new(NovelEntry {
            id: id.into(),
            source,
        }));
        self
    }

    pub fn init(&mut self) -> Result<()> {
        for source in &mut self.sources {
            source.init()?;
        }
        Ok(())
    }

    pub fn call(&mut self, source_id: &str, operation: &str, request_json: &str) -> Result<String> {
        let request: Value = serde_json::from_str(request_json)?;
        let preferences = request
            .get("preferences")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        let _context = crate::context::enter(source_id, preferences);
        let source = self
            .sources
            .iter_mut()
            .find(|source| source.id() == source_id)
            .ok_or_else(|| Error::new(format!("source {source_id:?} is not registered")))?;
        let value = source.call(operation, request)?;
        serde_json::to_string(&value).map_err(Error::from)
    }

    pub fn process_page_bytes(
        &mut self,
        source_id: &str,
        context_json: &str,
        bytes: &[u8],
        mime_type: Option<&str>,
    ) -> Result<Option<ProcessedImage>> {
        let context: Value = serde_json::from_str(context_json)?;
        let preferences = context
            .get("preferences")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        let _context = crate::context::enter(source_id, preferences);
        self.sources
            .iter_mut()
            .find(|source| source.id() == source_id)
            .ok_or_else(|| Error::new(format!("source {source_id:?} is not registered")))?
            .process_page_bytes(&context, bytes, mime_type)
    }

    pub fn process_video_resource(
        &mut self,
        source_id: &str,
        context_json: &str,
        bytes: &[u8],
        mime_type: Option<&str>,
    ) -> Result<Option<ProcessedMedia>> {
        let context: Value = serde_json::from_str(context_json)?;
        let preferences = context
            .get("preferences")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        let _context = crate::context::enter(source_id, preferences);
        self.sources
            .iter_mut()
            .find(|source| source.id() == source_id)
            .ok_or_else(|| Error::new(format!("source {source_id:?} is not registered")))?
            .process_video_resource(&context, bytes, mime_type)
    }
}

struct MangaEntry<T> {
    id: String,
    source: T,
}

impl<T: MangaSource> ErasedSource for MangaEntry<T> {
    fn id(&self) -> &str {
        &self.id
    }
    fn init(&mut self) -> Result<()> {
        self.source.init()
    }
    fn call(&mut self, operation: &str, request: Value) -> Result<Value> {
        match operation {
            "filters" => value(self.source.filters()?),
            "preferences" => value(self.source.preferences()?),
            "auth.status" => value(self.source.authentication_status()?),
            "auth.login" => value(self.source.authenticate(parse(request)?)?),
            "auth.logout" => value(self.source.logout()?),
            "home" | "manga.home" => value(self.source.home()?),
            "manga.list" => {
                let request: ListRequest = parse(request)?;
                value(
                    self.source
                        .listing(&request.listing, request.page, &request.filters)?,
                )
            }
            "manga.search" => {
                let request: SearchRequest = parse(request)?;
                value(
                    self.source
                        .search(&request.query, request.page, &request.filters)?,
                )
            }
            "manga.details" => value(self.source.details(field(&request, "manga")?)?),
            "manga.chapters" => value(self.source.chapters(field(&request, "manga")?)?),
            "manga.pages" => value(
                self.source
                    .pages(field(&request, "manga")?, field(&request, "chapter")?)?,
            ),
            "manga.item-url" => value(self.source.item_url(&field(&request, "manga")?)?),
            "manga.chapter-url" => value(
                self.source
                    .chapter_url(&field(&request, "manga")?, &field(&request, "chapter")?)?,
            ),
            "manga.handle-url" => value(self.source.handle_url(&string_field(&request, "url")?)?),
            "manga.prepare-chapter" => value(
                self.source
                    .prepare_chapter(&field(&request, "manga")?, field(&request, "chapter")?)?,
            ),
            "manga.resolve-page-image" => value(self.source.resolve_page_image(
                &field(&request, "manga")?,
                &field(&request, "chapter")?,
                &field(&request, "page")?,
            )?),
            "manga.process-page-image" => {
                let image = request
                    .get("imageBase64")
                    .and_then(Value::as_str)
                    .map(|value| {
                        use base64::Engine as _;
                        base64::engine::general_purpose::STANDARD
                            .decode(value)
                            .map_err(|error| Error::new(error.to_string()))
                    })
                    .transpose()?
                    .unwrap_or_default();
                value(self.source.process_page_image(
                    &field(&request, "manga")?,
                    &field(&request, "chapter")?,
                    &field(&request, "page")?,
                    &image,
                    request.get("mimeType").and_then(Value::as_str),
                )?)
            }
            "manga.alternate-covers" => {
                value(self.source.alternate_covers(&field(&request, "manga")?)?)
            }
            "manga.related" => value(self.source.related(&field(&request, "manga")?)?),
            "manga.migrate" => value(self.source.migrate(
                &field(&request, "manga")?,
                &string_field(&request, "targetSourceId")?,
            )?),
            other => self
                .source
                .dispatch(other, &request)?
                .ok_or_else(|| wrong_media(other, "manga")),
        }
    }

    fn process_page_bytes(
        &mut self,
        context: &Value,
        bytes: &[u8],
        mime_type: Option<&str>,
    ) -> Result<Option<ProcessedImage>> {
        self.source.process_page_image(
            &field(context, "manga")?,
            &field(context, "chapter")?,
            &field(context, "page")?,
            bytes,
            mime_type,
        )
    }
}

struct VideoEntry<T> {
    id: String,
    source: T,
}

impl<T: VideoSource> ErasedSource for VideoEntry<T> {
    fn id(&self) -> &str {
        &self.id
    }
    fn init(&mut self) -> Result<()> {
        self.source.init()
    }
    fn call(&mut self, operation: &str, request: Value) -> Result<Value> {
        match operation {
            "filters" => value(self.source.filters()?),
            "preferences" => value(self.source.preferences()?),
            "auth.status" => value(self.source.authentication_status()?),
            "auth.login" => value(self.source.authenticate(parse(request)?)?),
            "auth.logout" => value(self.source.logout()?),
            "home" | "video.home" => value(self.source.home()?),
            "video.list" => {
                let request: ListRequest = parse(request)?;
                value(
                    self.source
                        .listing(&request.listing, request.page, &request.filters)?,
                )
            }
            "video.search" => {
                let request: SearchRequest = parse(request)?;
                value(
                    self.source
                        .search(&request.query, request.page, &request.filters)?,
                )
            }
            "video.details" => value(self.source.details(field(&request, "item")?)?),
            "video.episodes" => value(self.source.episodes(field(&request, "item")?)?),
            "video.seasons" => value(self.source.seasons(field(&request, "item")?)?),
            "video.streams" => value(
                self.source
                    .streams(field(&request, "item")?, field(&request, "episode")?)?,
            ),
            "video.hosters" => value(
                self.source
                    .hosters(field(&request, "item")?, field(&request, "episode")?)?,
            ),
            "video.hoster-streams" => value(self.source.hoster_streams(
                field(&request, "item")?,
                field(&request, "episode")?,
                field(&request, "hoster")?,
            )?),
            "video.item-url" => value(self.source.item_url(&field(&request, "item")?)?),
            "video.episode-url" => value(
                self.source
                    .episode_url(&field(&request, "item")?, &field(&request, "episode")?)?,
            ),
            "video.handle-url" => value(self.source.handle_url(&string_field(&request, "url")?)?),
            other => self
                .source
                .dispatch(other, &request)?
                .ok_or_else(|| wrong_media(other, "video")),
        }
    }

    fn process_video_resource(
        &mut self,
        context: &Value,
        bytes: &[u8],
        mime_type: Option<&str>,
    ) -> Result<Option<ProcessedMedia>> {
        self.source.process_resource(context, bytes, mime_type)
    }
}

struct NovelEntry<T> {
    id: String,
    source: T,
}

impl<T: NovelSource> ErasedSource for NovelEntry<T> {
    fn id(&self) -> &str {
        &self.id
    }
    fn init(&mut self) -> Result<()> {
        self.source.init()
    }
    fn call(&mut self, operation: &str, request: Value) -> Result<Value> {
        match operation {
            "filters" => value(self.source.filters()?),
            "preferences" => value(self.source.preferences()?),
            "auth.status" => value(self.source.authentication_status()?),
            "auth.login" => value(self.source.authenticate(parse(request)?)?),
            "auth.logout" => value(self.source.logout()?),
            "home" | "novel.home" => value(self.source.home()?),
            "novel.list" => {
                let request: ListRequest = parse(request)?;
                value(
                    self.source
                        .listing(&request.listing, request.page, &request.filters)?,
                )
            }
            "novel.search" => {
                let request: SearchRequest = parse(request)?;
                value(
                    self.source
                        .search(&request.query, request.page, &request.filters)?,
                )
            }
            "novel.details" => value(self.source.details(field(&request, "item")?)?),
            "novel.chapters" => value(self.source.chapters(field(&request, "item")?)?),
            "novel.chapters-page" => value(
                self.source.chapters_page(
                    field(&request, "item")?,
                    request
                        .get("page")
                        .and_then(Value::as_u64)
                        .and_then(|page| u32::try_from(page).ok())
                        .unwrap_or(1),
                )?,
            ),
            "novel.text" => value(
                self.source
                    .text(field(&request, "item")?, field(&request, "chapter")?)?,
            ),
            "novel.item-url" => value(self.source.item_url(&field(&request, "item")?)?),
            "novel.chapter-url" => value(
                self.source
                    .chapter_url(&field(&request, "item")?, &field(&request, "chapter")?)?,
            ),
            "novel.handle-url" => value(self.source.handle_url(&string_field(&request, "url")?)?),
            other => self
                .source
                .dispatch(other, &request)?
                .ok_or_else(|| wrong_media(other, "novel")),
        }
    }
}

fn parse<T: DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(Error::from)
}

fn field<T: DeserializeOwned>(value: &Value, name: &str) -> Result<T> {
    serde_json::from_value(
        value
            .get(name)
            .cloned()
            .ok_or_else(|| Error::new(format!("request is missing {name:?}")))?,
    )
    .map_err(Error::from)
}

fn string_field(value: &Value, name: &str) -> Result<String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::new(format!("request is missing string {name:?}")))
}

fn value<T: Serialize>(value: T) -> Result<Value> {
    serde_json::to_value(value).map_err(Error::from)
}

fn wrong_media(operation: &str, media: &str) -> Error {
    Error::new(format!(
        "operation {operation:?} is not available on this {media} source"
    ))
}
