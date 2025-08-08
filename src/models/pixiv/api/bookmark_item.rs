use color_eyre::eyre::Ok;
use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream;
use serde::Deserialize;
use serde::Serialize;
use tagstudio_db::Entry;
use tagstudio_db::Library;

use crate::models::pixiv::utils::StringOrNum;
use crate::ColEyre;
use crate::ColEyreVal;
use crate::models::pixiv::PixivProvider;
use crate::models::pixiv::special_tags::PIXIV_DATA_IMPORT;

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
            return Ok(());
        }

        // Skip downloading if already downloaded
        if !overwrite_file && self.is_downloaded(lib).await? {
            return Ok(());
        }

        PixivProvider::download_illust_id(lib, self.id.number()?, overwrite_file).await
    }

    pub async fn is_downloaded(&self, lib: &Library) -> ColEyreVal<bool> {
        for page in 0..self.page_count {
            let entries = Entry::find_by_filename(
                &mut *lib.db.get().await?,
                &format!("illust_{}_p{page}.png", self.id),
            )
            .await?;

            for entry in entries {
                // Ignore this entry if it's unlinked
                if !entry.exists_on_disk(&mut *lib.db.get().await?).await? {
                    continue;
                }

                if entry.path == format!("Taguerine Downloads/Pixiv/illust_{}_p{page}.png", self.id)
                {
                    return Ok(true);
                }

                if entry
                    .has_tag(&mut *lib.db.get().await?, PIXIV_DATA_IMPORT)
                    .await?
                {
                    return Ok(true);
                }
            }
        }

        return Ok(false);
    }
}
