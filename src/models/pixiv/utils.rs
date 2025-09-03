use core::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

use crate::ColEyreVal;
use crate::models::pixiv::PixivProvider;

impl PixivProvider {
    pub fn parse_illust_id(url: &str) -> Option<&str> {
        if url.starts_with("https://www.pixiv.net/en/artworks/") {
            return url.split("/").last();
        }

        None
    }

    pub fn get_illust_url_from_id(id: impl Display) -> String {
        format!("https://www.pixiv.net/en/artworks/{id}")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum StringOrNum {
    String(String),
    Number(u64),
}

impl StringOrNum {
    pub fn number(&self) -> ColEyreVal<u64> {
        match self {
            Self::Number(val) => Ok(*val),
            Self::String(val) => Ok(val.parse()?),
        }
    }
}

impl Display for StringOrNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(val) => write!(f, "{val}")?,
            Self::String(val) => write!(f, "{val}")?,
        };

        Ok(())
    }
}
