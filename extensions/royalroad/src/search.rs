use kuchiki::{traits::TendrilSink, NodeRef};
use once_cell::sync::Lazy;
use quelle_core::prelude::*;
use quelle_glue::prelude::*;
use serde::Serialize;
use url::Url;

use crate::{RoyalRoad, META};

#[derive(InputField, Serialize, Debug)]
pub struct FilterOptions {
    title: TextField,
    keyword: TextField,
    author: TextField,
    genres: SelectField,
    tags_include: SelectField,
    tags_exclude: SelectField,
    warnings: SelectField,
    page_count: RangeField,
    rating: RangeField,
    status: SelectField,
    order_by: FieldGroup<OrderByField>,
    novel_type: ChoiceField,
}

#[derive(InputField, Serialize, Debug)]
pub struct OrderByField {
    by: ChoiceField,
    order: ChoiceField,
}

impl_to_abi_for_serde!(&FilterOptions);
impl_from_abi_for_serde!(FilterResult);

expose_filter!(RoyalRoad);
impl FilterSearch for RoyalRoad {
    type Options = FilterOptions;

    fn filter_options() -> &'static Self::Options {
        &FILTER_OPTIONS
    }

    fn filter_search_url(
        filter: <Self::Options as InputField>::Type,
        page: i32,
    ) -> Result<String, QuelleError> {
        FILTER_OPTIONS
            .verify_input(&filter)
            .map_err(QuelleError::FilterVerificationFailed)?;

        // unwrap: This url is static and tested to be valid
        let mut url = Url::parse("https://www.royalroad.com/fictions/search").unwrap();

        {
            let mut query = url.query_pairs_mut();

            query.append_pair("page", &format!("{page}"));

            const TAGS_ADD: &'static str = "tagsAdd";
            const TAGS_REMOVE: &'static str = "tagsRemove";

            if let Some(title) = filter.title {
                if !title.is_empty() {
                    query.append_pair("title", &title);
                }
            }

            if let Some(keyword) = filter.keyword {
                if !keyword.is_empty() {
                    query.append_pair("keyword", &keyword);
                }
            }

            if let Some(author) = filter.author {
                if !author.is_empty() {
                    query.append_pair("author", &author);
                }
            }

            if let Some(genres) = filter.genres {
                for genre in genres {
                    let name = if genre.remove { TAGS_REMOVE } else { TAGS_ADD };
                    query.append_pair(name, &genre.value);
                }
            }

            if let Some(tags) = filter.tags_include {
                for tag in tags {
                    query.append_pair(TAGS_ADD, &tag.value);
                }
            }

            if let Some(tags) = filter.tags_exclude {
                for tag in tags {
                    query.append_pair(TAGS_REMOVE, &tag.value);
                }
            }

            if let Some(warnings) = filter.warnings {
                for warning in warnings {
                    let name = if warning.remove {
                        TAGS_REMOVE
                    } else {
                        TAGS_ADD
                    };
                    query.append_pair(name, &warning.value);
                }
            }

            if let Some(page_count) = filter.page_count {
                if page_count.min > FILTER_OPTIONS.page_count.min {
                    query.append_pair("minPages", &format!("{}", page_count.min));
                }
                if page_count.max < FILTER_OPTIONS.page_count.max {
                    query.append_pair("maxPages", &format!("{}", page_count.max));
                }
            }

            if let Some(rating) = filter.rating {
                if rating.min > FILTER_OPTIONS.rating.min {
                    query.append_pair("minRating", &format!("{}", rating.min));
                }
                if rating.max > FILTER_OPTIONS.rating.max {
                    query.append_pair("maxRating", &format!("{}", rating.max));
                }
            }

            if let Some(statuses) = filter.status {
                for status in statuses {
                    query.append_pair("status", &status.value);
                }
            }

            if let Some(order_by) = filter.order_by.as_ref().map(|v| v.by.as_ref()).flatten() {
                query.append_pair("orderBy", order_by);
            }

            if let Some(dir) = filter.order_by.as_ref().map(|v| v.order.as_ref()).flatten() {
                query.append_pair("dir", dir);
            }

            if let Some(ty) = filter.novel_type {
                query.append_pair("type", &ty);
            }
        }

        Ok(url.into())
    }

    fn filter_search(
        filter: <Self::Options as InputField>::Type,
        page: i32,
    ) -> Result<Vec<BasicNovel>, QuelleError> {
        let url = Self::filter_search_url(filter, page)?;
        let response = Request::get(url.clone()).send()?;
        let doc = kuchiki::parse_html().one(response.text()?.unwrap());
        parse_search(url, doc)
    }
}

expose_text!(RoyalRoad);
impl TextSearch for RoyalRoad {
    fn text_search_url(query: String, page: i32) -> Result<String, QuelleError> {
        let url = format!("https://www.royalroad.com/fictions/search?title={query}&page={page}");
        Ok(url)
    }

    fn text_search(query: String, page: i32) -> Result<Vec<BasicNovel>, QuelleError> {
        let url = Self::text_search_url(query, page).unwrap();
        let response = Request::get(url.clone()).send()?;
        let doc = kuchiki::parse_html().one(response.text()?.unwrap());
        parse_search(url, doc)
    }
}

fn parse_search(url: String, doc: NodeRef) -> Result<Vec<BasicNovel>, QuelleError> {
    let mut novels = vec![];
    if let Ok(elements) = doc.select(".fiction-list-item") {
        for div in elements {
            let Some(a) = div.as_node().select_first(".fiction-title a").ok() else { continue };
            let Some(link) = a.get_attribute("href") else { continue };

            let cover = div
                .as_node()
                .select_first("img")
                .get_attribute("src")
                .map(|src| META.convert_into_absolute_url(src, Some(&url)))
                .transpose()?;

            let novel = BasicNovel {
                title: a.get_text(),
                url: META.convert_into_absolute_url(link, Some(&url))?,
                cover,
            };

            novels.push(novel);
        }
    }

    Ok(novels)
}

static FILTER_OPTIONS: Lazy<FilterOptions> = Lazy::new(|| {
    let tags = vec![
        Check::new("Anti-Hero Lead", "anti-hero_lead", false),
        Check::new("Artificial Intelligence", "artificial_intelligence", false),
        Check::new("Attractive Lead", "attractive_lead", false),
        Check::new("Cyberpunk", "cyberpunk", false),
        Check::new("Dungeon", "dungeon", false),
        Check::new("Dystopia", "dystopia", false),
        Check::new("Female Lead", "female_lead", false),
        Check::new("First Contact", "first_contact", false),
        Check::new("GameLit", "gamelit", false),
        Check::new("Gender Bender", "gender_bender", false),
        Check::new("Genetically Engineered", "genetically_engineered ", false),
        Check::new("Grimdark", "grimdark", false),
        Check::new("Hard Sci-fi", "hard_sci-fi", false),
        Check::new("Harem", "harem", false),
        Check::new("High Fantasy", "high_fantasy", false),
        Check::new("LitRPG", "litrpg", false),
        Check::new("Low Fantasy", "low_fantasy", false),
        Check::new("Magic", "magic", false),
        Check::new("Male Lead", "male_lead", false),
        Check::new("Martial Arts", "martial_arts", false),
        Check::new("Multiple Lead Characters", "multiple_lead", false),
        Check::new("Mythos", "mythos", false),
        Check::new("Non-Human Lead", "non-human_lead", false),
        Check::new("Portal Fantasy / Isekai", "summoned_hero", false),
        Check::new("Post Apocalyptic", "post_apocalyptic", false),
        Check::new("Progression", "progression", false),
        Check::new("Reader Interactive", "reader_interactive", false),
        Check::new("Reincarnation", "reincarnation", false),
        Check::new("Ruling Class", "ruling_class", false),
        Check::new("School Life", "school_life", false),
        Check::new("Secret Identity", "secret_identity", false),
        Check::new("Slice of Life", "slice_of_life", false),
        Check::new("Soft Sci-fi", "soft_sci-fi", false),
        Check::new("Space Opera", "space_opera", false),
        Check::new("Sports", "sports", false),
        Check::new("Steampunk", "steampunk", false),
        Check::new("Strategy", "strategy", false),
        Check::new("Strong Lead", "strong_lead", false),
        Check::new("Super Heroes", "super_heroes", false),
        Check::new("Supernatural", "supernatural", false),
        Check::new(
            "Technologically Engineered",
            "technologically_engineered",
            false,
        ),
        Check::new("Time Loop", "loop", false),
        Check::new("Time Travel", "time_travel", false),
        Check::new("Urban Fantasy", "urban_fantasy", false),
        Check::new("Villainous Lead", "villainous_lead", false),
        Check::new("Virtual Reality", "virtual_reality", false),
        Check::new("War and Military", "war_and_military", false),
        Check::new("Wuxia", "wuxia", false),
        Check::new("Xianxia", "xianxia", false),
    ];

    FilterOptions {
        title: TextField {
            title: String::from("Title"),
        },
        keyword: TextField {
            title: String::from("Keyword"),
        },
        author: TextField {
            title: String::from("Author name"),
        },
        genres: SelectField {
            title: String::from("Genres"),
            items: vec![
                Check::new("Action", "action", true),
                Check::new("Adventure", "adventure", true),
                Check::new("Comedy", "comedy", true),
                Check::new("Contemporary", "contemporary", true),
                Check::new("Drama", "drama", true),
                Check::new("Fantasy", "fantasy", true),
                Check::new("Historical", "historical", true),
                Check::new("Horror", "horror", true),
                Check::new("Mystery", "mystery", true),
                Check::new("Psychological", "psychological", true),
                Check::new("Romance", "romance", true),
                Check::new("Satire", "satire", true),
                Check::new("Sci-fi", "sci-fi", true),
                Check::new("Short Story", "one-shot", true),
                Check::new("Tragedy", "tragedy", true),
            ],
        },
        tags_include: SelectField {
            title: String::from("Only include matching all tags"),
            items: tags.clone(),
        },
        tags_exclude: SelectField {
            title: String::from("Exclude matching any tags"),
            items: tags,
        },
        warnings: SelectField {
            title: String::from("Content Warnings"),
            items: vec![
                Check::new("Profanity", "profanity", true),
                Check::new("Sexual Content", "sexuality", true),
                Check::new("Gore", "gore", true),
                Check::new("Gore", "gore", true),
                Check::new("Traumatising content", "traumatising", true),
                Check::new("AI-Assisted Content", "ai_assisted", true),
                Check::new("AI-Generated Content", "ai_generated", true),
            ],
        },
        page_count: RangeField {
            title: String::from("Number of Pages"),
            min: 0.0,
            max: 20000.0,
            div: 1.0,
        },
        rating: RangeField {
            title: String::from("Rating"),
            min: 0.0,
            max: 5.0,
            div: 0.1,
        },
        status: SelectField {
            title: String::from("Status"),
            items: vec![
                Check::new("All", "ALL", false),
                Check::new("Completed", "COMPELTED", false),
                Check::new("Dropped", "DROPPED", false),
                Check::new("Ongoing", "ONGOING", false),
                Check::new("Hiatus", "HAITUS", false),
                Check::new("Stub", "STUB", false),
            ],
        },
        order_by: FieldGroup {
            title: String::from("Order by"),
            fields: OrderByField {
                by: ChoiceField {
                    title: String::new(),
                    items: vec![
                        Choice::new("Relevence", "relevence"),
                        Choice::new("Popularity", "popularity"),
                        Choice::new("Average Rating", "rating"),
                        Choice::new("Last Update", "last_update"),
                        Choice::new("Number of Pages", "length"),
                        Choice::new("Views", "views"),
                        Choice::new("Title", "title"),
                        Choice::new("Author", "author"),
                    ],
                },
                order: ChoiceField {
                    title: String::new(),
                    items: vec![
                        Choice::new("Relevence", "relevence"),
                        Choice::new("Popularity", "popularity"),
                        Choice::new("Average Rating", "rating"),
                    ],
                },
            },
        },
        novel_type: ChoiceField {
            title: String::from("Type"),
            items: vec![
                Choice::new("All", "ALL"),
                Choice::new("Fan Fiction", "fanfiction"),
                Choice::new("Original", "original"),
            ],
        },
    }
});
