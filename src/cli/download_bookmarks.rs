use std::env::current_dir;

use clap::Parser;
use color_eyre::eyre::Context as _;
use futures::StreamExt;
use futures::TryStreamExt as _;
use futures::pin_mut;
use streamies::TryStreamies;
use tagstudio_db::Library;

use crate::models::pixiv::PixivProvider;

/// Add links to images based on their filename
#[derive(Parser, Debug, Clone)]
pub struct DownloadBookmarksCommand {
    /// The user id to download from
    user_id: u64,

    /// Download hidden bookmarks
    #[clap(long)]
    hidden: bool,

    /// Overwrite the downloaded files
    #[clap(short, long)]
    overwrite_file: bool,
}

impl DownloadBookmarksCommand {
    pub async fn run(&self) -> crate::ColEyre {
        let current_dir = current_dir().context("Couldn't get current working directory")?;
        let lib = Library::try_new(current_dir.clone()).context("Couldn't get the root library")?;
        let stream = PixivProvider::stream_bookmarks(self.user_id, self.hidden)
            .map_ok(async |bookmark| bookmark.download(&lib, self.overwrite_file).await)
            .extract_future_ok()
            .buffer_unordered(1)
            .flatten_result_ok();

        pin_mut!(stream);

        while let Some(_val) = stream.try_next().await? {}

        Ok(())
    }
}
