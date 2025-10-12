use std::collections::HashMap;

use crate::models::pixiv::api::illust_body::IllustBody;
use crate::models::pixiv::api::illust_pages::IllustPage;

#[derive(Debug, Default)]
pub struct IllustCache {
    inner: HashMap<u64, IllustPage>,
}

// impl IllustCache {
//     pub fn insert(&mut self, data: IllustBody) {
//         let id: u64 = self.illust_id.parse()?;
//         self.inner.insert(data.id, data);
//     }

//     pub fn get(&self, illust_id: i64) -> Option<&IllustBody> {
//         self.inner.get(&(illust_id, page_id))
//     }
// }
