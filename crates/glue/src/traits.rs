use quelle_core::prelude::*;

/// This is the minimal functionality required for each extension/source.
///
/// The trait should be exposed to wasm abi using [`expose_basic`]
///
/// ## Example
///
/// ```ignore
/// struct ExtensionName;
/// expose_basic!(ExtensionName);
/// ```
pub trait FetchBasic {
    /// Retrieve the meta information about a novel and its chapter list.
    fn fetch_novel(url: String) -> Result<Novel, QuelleError>;

    /// Fetch the content of the chapter as html text
    ///
    /// The returned html should be cleaned of all unnecessary content and tags.
    /// This includes empty elements, ad elements, and most tags.
    fn fetch_chapter_content(url: String) -> Result<Content, QuelleError>;
}

/// The macro used to export [FetchBasic] to wasm abi
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

/// This trait adds popular search functionality to an extension/source
///
/// The trait should be exposed to wasm abi using [`expose_popular`]
///
/// ## Example
///
/// ```ignore
/// struct ExtensionName;
/// expose_popular!(ExtensionName);
/// ```
pub trait PopularSearch {
    /// Construct a url pointing to the browseable popular page
    fn popular_url(page: i32) -> String;

    /// Search for the popular/trending novels
    fn popular(page: i32) -> Result<Vec<BasicNovel>, QuelleError>;
}

/// The macro used to export [PopularSearch] to wasm abi
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

/// This trait adds text search functionality to an extension/source
///
/// The trait should be exposed to wasm abi using [`expose_text`]
///
/// ## Example
///
/// ```ignore
/// struct ExtensionName;
/// expose_text!(ExtensionName);
/// ```
pub trait TextSearch {
    /// Construct a url pointing to a webpage with the given arguments
    ///
    /// In the case where a website url cannot be constructed with query
    /// (ex: the novels are fetched with an api call), the next
    /// closest webpage should be returned.
    /// This can be the search page (ex: https://example.com/search) or if
    /// no such page exists base url may be returned.
    fn text_search_url(query: String, page: i32) -> Result<String, QuelleError>;

    /// Search for novels with the given query and page
    ///
    /// See [FilterSearch::filter_search] for more complex filtering implementations
    fn text_search(query: String, page: i32) -> Result<Vec<BasicNovel>, QuelleError>;
}

/// The macro used to export [TextSearch] to wasm abi
#[macro_export]
macro_rules! expose_text {
    ($name:ident) => {
        #[quelle_glue::prelude::expose]
        pub fn text_search(query: String, page: i32) -> Result<Vec<BasicNovel>, QuelleError> {
            <$name as $crate::traits::TextSearch>::text_search(query, page)
        }
    };
}

/// This trait adds filter search functionality to an extension/source
///
/// The trait should be exposed to wasm abi using [`expose_filter`]
///
/// ## Example
///
/// ```ignore
/// struct ExtensionName;
/// expose_filter!(ExtensionName);
/// ```
pub trait FilterSearch
where
    <Self as FilterSearch>::Options: quelle_core::filter::InputField,
{
    /// The options that are available for filtering
    ///
    /// [FilterSearch] constrains the options to implement [InputField](quelle_core::filter::InputField).
    /// This provides validation and filter parsing.
    /// [InputField] can be derived given when all its field are also [InputField].
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use quelle_core::filter::{InputField, TextField};
    /// # use quelle_glue_derive::InputField;
    ///
    /// #[derive(InputField)]
    /// struct FilterOptions {
    ///     title: TextField,
    /// }
    /// ```
    type Options;

    /// A getter for [`Self::Options`] available for filtering
    fn filter_options() -> &'static Self::Options;

    /// Construct a url pointing to a webpage with the given arguments
    ///
    /// In the case where a website url cannot be constructed with the filter
    /// options (ex: the novels are fetched with an api call), the next
    /// closest webpage should be returned.
    /// This can be the search page (ex: https://example.com/search) or if
    /// no such page exists base url may be returned.
    fn filter_search_url(
        filter: <Self::Options as InputField>::Type,
        page: i32,
    ) -> Result<String, QuelleError>;

    /// Search for novels with the given filter and page
    fn filter_search(
        filter: <Self::Options as InputField>::Type,
        page: i32,
    ) -> Result<Vec<BasicNovel>, QuelleError>;
}

/// The macro used to export [FilterSearch] to wasm abi
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
