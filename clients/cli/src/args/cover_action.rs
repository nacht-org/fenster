use std::str::FromStr;

/// Defines how the novel cover download should be handled
#[derive(Clone, Debug, Default)]
pub enum CoverAction {
    /// Dynamic will skip the cover download if cover is already downloaded.
    #[default]
    Dynamic,

    /// Force action will always download the cover
    Force,

    /// Ignore will always skip the download.
    Ignore,
}

impl FromStr for CoverAction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dynamic" => Ok(CoverAction::Dynamic),
            "force" => Ok(CoverAction::Force),
            "ignore" => Ok(CoverAction::Ignore),
            _ => Err("unable to parse unknown cover action"),
        }
    }
}
