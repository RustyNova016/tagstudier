use crate::models::pixiv::PixivProvider;

impl PixivProvider {
    pub fn parse_illust_id(url: &str) -> Option<&str> {
        if url.starts_with("https://www.pixiv.net/en/artworks/") {
            return url.split("/").last();
        }

        None
    }
}
