use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream;
use serde::Deserialize;
use serde::Serialize;
use tagstudio_db::Entry;
use tagstudio_db::Library;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::models::pixiv::PixivProvider;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkItem {
    pub id: String,
    pub title: String,
    pub user_name: String,
    pub page_count: u64,
    pub illust_type: u64,
}
impl BookmarkItem {
    pub async fn download(&self, lib: &Library, overwrite_file: bool) -> ColEyre {
        // Skip downloading if already downloaded
        if !overwrite_file && self.is_downloaded(lib).await? {
            return Ok(());
        }

        PixivProvider::download_illust_id(lib, self.id.parse()?, overwrite_file).await
    }

    pub async fn is_downloaded(&self, lib: &Library) -> ColEyreVal<bool> {
        stream::iter(0..self.page_count)
            .map(async |page| {
                Ok(!Entry::find_by_path(
                    &mut *lib.db.get().await?,
                    &format!("Tagerine Downloads/Pixiv/illust_{}_p{page}.png", self.id),
                )
                .await?
                .is_empty())
            })
            .buffer_unordered(8)
            .try_all(|b| async move { b })
            .await
    }
}
