use crate::{
    bindings::{
        exports::quelle::extension::{
            instance::{self, GuestSource},
            meta,
        },
        quelle::core::{novel, source},
    },
    ExtensionMeta, ExtensionOptions, ExtensionSource,
};

impl<T> meta::Guest for T
where
    T: ExtensionMeta,
{
    fn extension_info() -> meta::SourceMeta {
        T::info().into()
    }

    fn setup(options: meta::ExtensionOptions) -> Result<(), String> {
        T::setup(options.into()).into()
    }
}

impl From<quelle_core::data::Meta> for meta::SourceMeta {
    fn from(meta: quelle_core::data::Meta) -> Self {
        meta::SourceMeta {
            id: meta.id,
            name: meta.name,
            langs: meta.langs,
            version: meta.version,
            base_urls: meta.base_urls,
            rds: meta.rds.into_iter().map(Into::into).collect(),
            attrs: meta.attrs.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<quelle_core::data::ReadingDirection> for source::ReadingDirection {
    fn from(direction: quelle_core::data::ReadingDirection) -> Self {
        match direction {
            quelle_core::data::ReadingDirection::Ltr => source::ReadingDirection::Ltr,
            quelle_core::data::ReadingDirection::Rtl => source::ReadingDirection::Rtl,
        }
    }
}

impl From<quelle_core::data::Attribute> for source::SourceAttr {
    fn from(attribute: quelle_core::data::Attribute) -> Self {
        match attribute {
            quelle_core::data::Attribute::Fanfiction => source::SourceAttr::Fanfiction,
        }
    }
}

impl From<meta::ExtensionOptions> for ExtensionOptions {
    fn from(value: meta::ExtensionOptions) -> Self {
        Self {}
    }
}

impl<T: 'static> GuestSource for T
where
    T: ExtensionSource,
{
    fn new(options: instance::SourceOptions) -> Self {
        T::new(options.into())
    }

    fn novel_info(&self, url: String) -> Result<instance::Novel, String> {
        self.novel_info(&url).map(Into::into)
    }

    fn chapter_content(&self, url: String) -> Result<instance::ChapterContent, String> {
        self.chapter_content(&url).map(Into::into)
    }
}

impl From<quelle_core::data::Novel> for instance::Novel {
    fn from(novel: quelle_core::data::Novel) -> Self {
        Self {
            url: novel.url,
            authors: novel.authors,
            title: novel.title,
            cover: novel.cover,
            description: novel.description,
            volumes: novel.volumes.into_iter().map(Into::into).collect(),
            metadata: novel.metadata.into_iter().map(Into::into).collect(),
            status: novel.status.into(),
            langs: novel.langs,
        }
    }
}

impl From<quelle_core::data::Volume> for novel::Volume {
    fn from(volume: quelle_core::data::Volume) -> Self {
        Self {
            name: volume.name,
            index: volume.index,
            chapters: volume.chapters.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<quelle_core::data::Chapter> for novel::Chapter {
    fn from(chapter: quelle_core::data::Chapter) -> Self {
        Self {
            title: chapter.title,
            url: chapter.url,
            index: chapter.index,
            updated_at: None,
        }
    }
}

impl From<quelle_core::data::Metadata> for novel::Metadata {
    fn from(value: quelle_core::data::Metadata) -> Self {
        Self {
            name: value.name,
            value: value.value,
            ns: value.ns.into(),
            others: value.others.into_iter().collect(),
        }
    }
}

impl From<quelle_core::data::Namespace> for novel::Namespace {
    fn from(value: quelle_core::data::Namespace) -> Self {
        match value {
            quelle_core::data::Namespace::DC => novel::Namespace::Dc,
            quelle_core::data::Namespace::OPF => novel::Namespace::Opf,
        }
    }
}

impl From<quelle_core::data::NovelStatus> for novel::NovelStatus {
    fn from(status: quelle_core::data::NovelStatus) -> Self {
        match status {
            quelle_core::data::NovelStatus::Ongoing => novel::NovelStatus::Ongoing,
            quelle_core::data::NovelStatus::Hiatus => novel::NovelStatus::Hiatus,
            quelle_core::data::NovelStatus::Completed => novel::NovelStatus::Completed,
            quelle_core::data::NovelStatus::Stub => novel::NovelStatus::Stub,
            quelle_core::data::NovelStatus::Dropped => novel::NovelStatus::Dropped,
            quelle_core::data::NovelStatus::Unknown => novel::NovelStatus::Unknown,
        }
    }
}

impl From<quelle_core::data::Content> for instance::ChapterContent {
    fn from(content: quelle_core::data::Content) -> Self {
        Self { data: content.data }
    }
}

impl From<instance::SourceOptions> for crate::SourceOptions {
    fn from(options: instance::SourceOptions) -> Self {
        Self {}
    }
}
