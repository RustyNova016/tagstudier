use serde::Deserialize;
use serde::Serialize;

use crate::models::pixiv::api::bookmark_item::BookmarkItem;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkResponse {
    pub works: Vec<BookmarkItem>,
    pub total: u64,
}

impl BookmarkResponse {
    // pub fn download(&self, lib: &Library, overwrite_file: bool) -> impl Stream<Item = ColEyre> {
    //     stream::iter(&self.works)
    //         .map(move |work| async move { work.download(lib, overwrite_file).await })
    //         .buffer_unordered(1)
    // }
}
