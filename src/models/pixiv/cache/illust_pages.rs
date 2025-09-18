use std::collections::HashMap;

use crate::models::pixiv::api::illust_pages::IllustPage;

#[derive(Debug, Default)]
pub struct IllustPageCache {
    inner: HashMap<(u64, u64), IllustPage>,
}

impl IllustPageCache {
    pub fn insert(&mut self, illust_id: u64, page_id: u64, data: IllustPage) {
        self.inner.insert((illust_id, page_id), data);
    }

    pub fn get(&self, illust_id: u64, page_id: u64) -> Option<&IllustPage> {
        self.inner.get(&(illust_id, page_id))
    }
}
