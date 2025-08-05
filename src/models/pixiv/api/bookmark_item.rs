use serde::Deserialize;
use serde::Serialize;
use tagstudio_db::Library;

use crate::ColEyre;
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
        PixivProvider::download_illust_id(lib, self.id.parse()?, overwrite_file).await
    }
}
