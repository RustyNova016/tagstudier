use core::fmt::Display;

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
