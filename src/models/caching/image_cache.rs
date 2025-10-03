use std::collections::HashMap;
use std::sync::LazyLock;

use futures::lock::Mutex;
use image::DynamicImage;
use image::ImageReader;
use image::imageops::FilterType;
use tagstudio_db::Entry;
use tagstudio_db::Library;

use crate::ColEyre;

pub static IMAGE_CACHE: LazyLock<ImageCache> = LazyLock::new(|| ImageCache::default());

#[derive(Debug, Default)]
pub struct ImageCache {
    cache: Mutex<HashMap<i64, DynamicImage>>,
}

impl ImageCache {
    async fn load_image(&self, lib: &Library, id: i64) -> ColEyre<DynamicImage> {
        let conn = &mut *lib.db.get().await?;
        let entry = Entry::find_by_id(conn, id).await?.unwrap();
        let path = entry.get_global_path(conn).await?;

        let img = ImageReader::open(path)?.decode()?;
        let img = img.resize(500, 500, FilterType::CatmullRom);

        Ok(img)
    }

    pub async fn get_or_init(&self, lib: &Library, id: i64) -> ColEyre<DynamicImage> {
        let mut cache = self.cache.lock().await;
        match cache.get(&id) {
            Some(val) => Ok(val.clone()),
            None => Ok(cache
                .entry(id)
                .insert_entry(self.load_image(lib, id).await?)
                .get()
                .clone()),
        }
    }
}
