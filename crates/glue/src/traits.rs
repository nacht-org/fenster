use quelle_core::prelude::*;

pub trait FetchBasic {
    fn fetch_novel(url: String) -> Result<Novel, QuelleError>;
    fn fetch_chapter_content(url: String) -> Result<Content, QuelleError>;
}

#[macro_export]
macro_rules! expose_basic {
    ($name:ident) => {
        #[quelle_glue::prelude::expose]
        pub fn fetch_novel(url: String) -> Result<Novel, QuelleError> {
            <$name as $crate::traits::FetchBasic>::fetch_novel(url)
        }

        #[quelle_glue::prelude::expose]
        pub fn fetch_chapter_content(url: String) -> Result<Content, QuelleError> {
            <$name as $crate::traits::FetchBasic>::fetch_chapter_content(url)
        }
    };
}

pub trait PopularSearch {
    fn popular_url(page: i32) -> String;
    fn popular(page: i32) -> Result<Vec<BasicNovel>, QuelleError>;
}

#[macro_export]
macro_rules! expose_popular {
    ($name:ident) => {
        #[quelle_glue::prelude::expose]
        pub fn popular_url(page: i32) -> String {
            <$name as $crate::traits::PopularSearch>::popular_url(page)
        }

        #[quelle_glue::prelude::expose]
        pub fn popular(page: i32) -> Result<Vec<BasicNovel>, QuelleError> {
            <$name as $crate::traits::PopularSearch>::popular(page)
        }
    };
}

pub trait TextSearch {
    fn text_search_url(query: String, page: i32) -> Result<String, QuelleError>;
    fn text_search(query: String, page: i32) -> Result<Vec<BasicNovel>, QuelleError>;
}

#[macro_export]
macro_rules! expose_text {
    ($name:ident) => {
        #[quelle_glue::prelude::expose]
        pub fn text_search(query: String, page: i32) -> Result<Vec<BasicNovel>, QuelleError> {
            <$name as $crate::traits::TextSearch>::text_search(query, page)
        }
    };
}

pub trait FilterSearch
where
    <Self as FilterSearch>::Options: quelle_core::filter::InputField,
{
    type Options;
    fn filter_options() -> &'static Self::Options;
    fn filter_search_url(
        filter: <Self::Options as InputField>::Type,
        page: i32,
    ) -> Result<String, QuelleError>;
    fn filter_search(
        filter: <Self::Options as InputField>::Type,
        page: i32,
    ) -> Result<Vec<BasicNovel>, QuelleError>;
}

#[macro_export]
macro_rules! expose_filter {
    ($name:ident) => {
        #[quelle_glue::prelude::expose]
        pub fn filter_options() -> &'static <$name as $crate::traits::FilterSearch>::Options {
            <$name as $crate::traits::FilterSearch>::filter_options()
        }

        #[quelle_glue::prelude::expose]
        pub fn filter_search_url(
            filter: <<$name as $crate::traits::FilterSearch>::Options as InputField>::Type,
            page: i32,
        ) -> Result<String, QuelleError> {
            <$name as $crate::traits::FilterSearch>::filter_search_url(filter, page)
        }

        #[quelle_glue::prelude::expose]
        pub fn filter_search(
            filter: <<$name as $crate::traits::FilterSearch>::Options as InputField>::Type,
            page: i32,
        ) -> Result<Vec<BasicNovel>, QuelleError> {
            <$name as $crate::traits::FilterSearch>::filter_search(filter, page)
        }
    };
}
