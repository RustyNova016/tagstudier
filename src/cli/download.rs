use std::env::current_dir;

use clap::Parser;
use color_eyre::eyre::Context;
use tagstudio_db::Library;

use crate::models::pixiv::PixivProvider;

/// Add links to images based on their filename
#[derive(Parser, Debug, Clone)]
pub struct DownloadCommand {
    /// The url to download
    url: String,

    #[clap(short, long)]
    overwrite_image: bool,
}

impl DownloadCommand {
    pub async fn run(&self) -> crate::ColEyre {
        let current_dir = current_dir().context("Couldn't get current working directory")?;
        let lib = Library::try_new(current_dir.clone()).context("Couldn't get the root library")?;
        PixivProvider::download_illust_url(&lib, &self.url, self.overwrite_image).await?;

        Ok(())
    }
}
