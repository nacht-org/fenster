#[macro_export]
macro_rules! define_meta {
    (
        let $var:ident = {
            id: $id:literal,
            name: $name:literal,
            langs: [$($lang:literal),+],
            base_urls: [$($base_url:literal),+],
            rds: [$($rd:ident),+],
            attrs: [$($attr:ident),*],
        };
    ) => {
        static $var: Lazy<Meta> = Lazy::new(|| Meta {
            id: String::from($id),
            name: String::from($name),
            langs: vec![$(String::from($lang)),+],
            version: String::from(env!("CARGO_PKG_VERSION")),
            base_urls: vec![$(String::from($base_url)),+],
            rds: vec![$(ReadingDirection::$rd),+],
            attrs: vec![$(Attribute::$attr),*],
        });


        #[expose]
        pub fn meta() -> &'static Meta {
            &$var
        }
    };
}

pub use define_meta;
