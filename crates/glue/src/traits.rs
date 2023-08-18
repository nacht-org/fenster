use quelle_core::prelude::{BasicNovel, QuelleError};

pub trait Popular {
    fn popular_url(page: i32) -> String;
    fn popular(page: i32) -> Result<Vec<BasicNovel>, QuelleError>;
}

#[macro_export]
macro_rules! expose_popular {
    ($name:ident) => {
        #[quelle_glue::prelude::expose]
        pub fn popular_url(page: i32) -> String {
            <$name as Popular>::popular_url(page)
        }

        #[quelle_glue::prelude::expose]
        pub fn popular(page: i32) -> Result<Vec<BasicNovel>, QuelleError> {
            <$name as Popular>::popular(page)
        }
    };
}
