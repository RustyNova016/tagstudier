use core::future::ready;

use color_eyre::eyre::Ok;
use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream;
use serde::Deserialize;
use serde::Serialize;
use streamies::TryStreamies;
use tagstudio_db::Entry;
use tagstudio_db::Library;
use tracing::debug;

use crate::exts::tagstudio_db_ext::entry::EntryExt as _;
use crate::ColEyre;
use crate::ColEyreVal;
use crate::models::pixiv::PixivProvider;
use crate::models::pixiv::special_tags::PIXIV_DATA_IMPORT;
use crate::models::pixiv::utils::StringOrNum;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkItem {
    pub id: StringOrNum,
    pub title: String,
    pub user_name: String,
    pub page_count: u64,
    pub illust_type: u64,
}

impl BookmarkItem {
    pub async fn download(&self, lib: &Library, overwrite_file: bool) -> ColEyre {
        // Skip if the illust is removed
        if &self.title == "-----" {
            debug!("Skipping download of id `{}: Deleted`", self.id);
            return Ok(());
        }

        // Skip downloading if already downloaded
        if !overwrite_file && self.is_downloaded(lib).await? {
            debug!("Skipping download of id `{}: Already downloaded`", self.id);
            return Ok(());
        }

        PixivProvider::download_illust_id(lib, self.id.number()?, overwrite_file).await
    }

    pub async fn is_downloaded(&self, lib: &Library) -> ColEyreVal<bool> {
        let id = self.id.number()?;

        let missing_data = stream::iter(0..self.page_count)
            .map(async |page| {
                Entry::find_downloaded_pixiv_entries(lib, id, page)
                    .await
                    .map(|entries| entries.is_empty())
            })
            .buffer_unordered(8)
            .try_any(|empty| ready(empty))
            .await?;

        Ok(!missing_data)
    }
}
