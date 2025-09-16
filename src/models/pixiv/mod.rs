pub mod models;
use color_eyre::eyre::OptionExt;
use tagstudio_db::models::library::Library;

pub mod api;
pub mod api_requests;
pub mod http_clients;
pub mod special_tags;
pub mod utils;

// pub static PIXIV_PROVIDER: PixivProvider = PixivProvider {};

pub struct PixivProvider {}

impl PixivProvider {
    pub async fn download_illust_url(
        lib: &Library,
        url: &str,
        overwrite_file: bool,
    ) -> color_eyre::Result<()> {
        Self::download_illust_id(
            lib,
            Self::parse_illust_id(url)
                .ok_or_eyre("Couldn't parse the url for the illust ID")?
                .parse()?,
            overwrite_file,
        )
        .await?;

        Ok(())
    }

    pub async fn download_illust_id(
        lib: &Library,
        id: u64,
        overwrite_file: bool,
    ) -> color_eyre::Result<()> {
        let illust = Self::fetch_illust(id).await?;
        illust.download_images(lib, overwrite_file).await?;

        Ok(())
    }
}

impl Default for PixivProvider {
    fn default() -> Self {
        PixivProvider {}
    }
}
