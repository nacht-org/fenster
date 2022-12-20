use std::{ops::RangeInclusive, str::FromStr};

use anyhow::{anyhow, bail};

#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct DownloadRange(pub RangeInclusive<usize>);

impl FromStr for DownloadRange {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("..=") {
            let (start_in, end_in) = match s.split_once("..=") {
                Some(v) => v,
                None => bail!("must contain only a single sequence of '..'"),
            };

            let start_in = start_in.parse::<usize>()?;
            let end_in = end_in.parse::<usize>()?;

            return Ok(DownloadRange(start_in..=end_in));
        } else if s.contains("..") {
            let (start_in, end_ex) = match s.split_once("..") {
                Some(v) => v,
                None => bail!("must contain only a single sequence of '..'"),
            };

            let start_in = start_in.parse::<usize>()?;
            let end_ex = end_ex.parse::<usize>()?;
            let end_in = end_ex - 1;

            return Ok(DownloadRange(start_in..=end_in));
        } else if let Ok(value) = s.parse::<usize>() {
            return Ok(DownloadRange(value..=value));
        }

        Err(anyhow!("unsupported format (ex: 1, 0..10, 0..=10)"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_download_range() -> anyhow::Result<()> {
        assert_eq!(DownloadRange::from_str("1")?, DownloadRange(1..=1));
        assert_eq!(DownloadRange::from_str("10")?, DownloadRange(10..=10));
        assert_eq!(DownloadRange::from_str("40..100")?, DownloadRange(40..=99));
        assert_eq!(DownloadRange::from_str("10..=20")?, DownloadRange(10..=20));
        Ok(())
    }
}
