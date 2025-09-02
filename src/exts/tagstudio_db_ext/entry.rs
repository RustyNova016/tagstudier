use std::path::PathBuf;

use color_eyre::eyre::Ok;
use extend::ext;
use futures::StreamExt as _;
use futures::TryStreamExt;
use futures::stream;
use itertools::Itertools;
use streamies::Streamies;
use streamies::TryStreamies;
use tagstudio_db::Entry;
use tagstudio_db::models::library::Library;
use tagstudio_db::query::Queryfragments;

use crate::ColEyreVal;
use crate::exts::path::PathExt;
use crate::models::pixiv::special_tags::PIXIV_DATA_IMPORT;

#[ext]
pub impl Entry {
    /// Find the entries corresponding to a pixiv illust page
    ///
    /// This means:
    /// - It has a name with the format `illust_{id}_p{page}.png`
    /// - It exist on disk
    /// - It has the "Pixiv: Imported Data" tag
    async fn find_downloaded_pixiv_entries(
        lib: &Library,
        illust_id: u64,
        page: u64,
    ) -> ColEyreVal<Vec<Entry>> {
        // TODO(Perf): Make it a stream

        let entries = Entry::find_by_filename(
            &mut *lib.db.get().await?,
            &format!("illust_{illust_id}_p{page}.png"),
        )
        .await?;

        stream::iter(entries)
            .map(async |entry| {
                if !entry.exists_on_disk(&mut *lib.db.get().await?).await? {
                    return Ok(None);
                }

                if !entry
                    .match_tag(&mut *lib.db.get().await?, PIXIV_DATA_IMPORT)
                    .await?
                {
                    return Ok(None);
                }

                Ok(Some(entry))
            })
            .buffered(8)
            .try_filter_map(|entry| async move { Ok(entry) })
            .try_collect_vec()
            .await
    }
}
