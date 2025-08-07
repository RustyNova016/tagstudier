use std::path::PathBuf;

use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use tagstudio_db::Entry;
use tagstudio_db::Library;
use tracing::debug;

use crate::ColEyreVal;
use crate::exts::path::PathExt as _;
use crate::exts::tagstudio_db_ext::library::LibraryExt;
use crate::models::pixiv::api::illust_urls::IllustUrls;
use crate::models::pixiv::http_clients::IPXIM_HTTP_CLIENT;
use crate::models::pixiv::http_clients::PIXIV_RATE_LIMIT;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IllustPage {
    pub urls: IllustUrls,
    pub width: u64,
    pub height: u64,
}

impl IllustPage {
    pub fn filename(illust_id: u64, page: u64) -> String {
        format!("illust_{illust_id}_p{page}.png")
    }

    pub fn library_path(illust_id: u64, page: u64) -> String {
        format!(
            "Tagerine Downloads/Pixiv/{}",
            Self::filename(illust_id, page)
        )
    }

    pub fn full_path(lib: &Library, illust_id: u64, page: u64) -> PathBuf {
        let pixiv_downloads = lib.get_download_folder().join("Pixiv");
        pixiv_downloads.create_directory_if_not_exist().unwrap();
        pixiv_downloads.join(Self::filename(illust_id, page))
    }

    pub async fn download(
        &self,
        lib: &Library,
        illust_id: u64,
        page: u64,
        overwrite_file: bool,
    ) -> ColEyreVal<Option<PathBuf>> {
        let Some(url) = &self.urls.original else {
            return Ok(None);
        };

        let file_path = Self::full_path(lib, illust_id, page);

        // Check if the file is already there, and if we don't overwrite, skip.
        if file_path.try_exists()? {
            if overwrite_file {
                file_path.delete_if_exists()?
            } else {
                return Ok(Some(file_path));
            }
        }

        // Then fetch the image
        PIXIV_RATE_LIMIT.until_ready().await;
        debug!("Fetching image: {url}");
        let img_bytes = IPXIM_HTTP_CLIENT.get(url).send().await?.bytes().await?;

        let image = image::load_from_memory(&img_bytes)?;
        file_path.create_file_if_not_exist()?;
        image.save(&file_path)?;

        Ok(Some(file_path))
    }

    async fn add_downloaded_file_entry(
        lib: &Library,
        illust_id: u64,
        page: u64,
    ) -> ColEyreVal<Vec<Entry>> {
        let conn = &mut *lib.db.get().await?;
        let entries = Entry::find_by_path(conn, &Self::library_path(illust_id, page)).await?;
        let folder = lib.get_root_db_folder(conn).await?;

        if entries.is_empty() {
            let entry = Entry {
                id: 0,
                filename: Self::filename(illust_id, page),
                folder_id: folder.id,
                path: Self::library_path(illust_id, page),
                suffix: "png".to_string(),
                date_added: Some(Utc::now().naive_local()),
                date_created: None,
                date_modified: None,
            };

            let entry = entry.insert(conn).await?;

            return Ok(vec![entry]);
        }

        Ok(entries)
    }

    pub async fn download_and_insert(
        &self,
        lib: &Library,
        illust_id: u64,
        page: u64,
        overwrite_file: bool,
    ) -> ColEyreVal<Vec<Entry>> {
        self.download(lib, illust_id, page, overwrite_file).await?;
        Self::add_downloaded_file_entry(lib, illust_id, page).await
    }
}
