use color_eyre::eyre::Context as _;
use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Acquire;
use streamies::TryStreamies;
use tagstudio_db::Entry;
use tagstudio_db::Library;
use tagstudio_db::Tag;
use tagstudio_db::query::Queryfragments;
use tagstudio_db::query::eq_field::EqField;
use tagstudio_db::query::eq_field::FieldValue;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::constants::AI_TAG;
use crate::models::pixiv::PixivProvider;
use crate::models::pixiv::api::illust_tag::IllustTags;
use crate::models::pixiv::api::illust_urls::IllustUrls;
use crate::models::pixiv::special_tags::PIXIV_DATA_IMPORT;
use crate::models::pixiv::special_tags::PIXIV_NO_DATA_UPDATE;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IllustBody {
    pub illust_id: String,
    pub illust_title: String,
    pub illust_comment: String,

    pub user_name: String,
    pub page_count: u64,
    pub tags: IllustTags,

    /// Whether the work is AI or not. 1 -> Not AI, 2 -> AI
    pub ai_type: u8,

    pub urls: IllustUrls,
}

impl IllustBody {
    pub async fn apply_to_entry(
        &self,
        conn: &mut sqlx::SqliteConnection,
        entry: &Entry,
    ) -> ColEyre {
        self.add_data_to_entry(conn, entry)
            .await
            .context("Error while adding data to an entry")?;
        self.add_tag_to_entry(conn, entry).await?;
        Ok(())
    }

    pub async fn add_data_to_entry(
        &self,
        conn: &mut sqlx::SqliteConnection,
        entry: &Entry,
    ) -> ColEyre {
        let mut trans = conn.begin().await?;

        // Check if the update is authorized
        if entry.has_tag(&mut trans, PIXIV_NO_DATA_UPDATE).await? {
            return Ok(());
        }

        entry
            .add_text_field(&mut trans, "TITLE", &self.illust_title)
            .await
            .context(format!("Error while adding title to entry {}", entry.id))?;
        entry
            .add_text_field(&mut trans, "ARTIST", &self.user_name)
            .await?;
        entry
            .add_text_field(
                &mut trans,
                "URL",
                &format!("https://www.pixiv.net/en/artworks/{}", self.illust_id),
            )
            .await?;

        if self.ai_type == 2 {
            let ai_gen_tag = Tag::get_by_name_or_insert_new(&mut trans, AI_TAG)
                .await
                .unwrap();

            entry.add_tags(&mut trans, &ai_gen_tag).await.unwrap();
        }

        let pixiv_import_tag = Tag::get_by_name_or_insert_new(&mut trans, PIXIV_DATA_IMPORT)
            .await
            .unwrap();

        entry.add_tags(&mut trans, &pixiv_import_tag).await?;

        trans.commit().await?;

        Ok(())
    }

    pub async fn add_tag_to_entry(
        &self,
        conn: &mut sqlx::SqliteConnection,
        entry: &Entry,
    ) -> ColEyre {
        let mut trans = conn.begin().await?;

        for tag in &self.tags.tags {
            tag.add_to_entry(&mut trans, entry).await?
        }

        trans.commit().await?;

        Ok(())
    }

    /// Add the data to all the entries that are associated with this illust
    pub async fn apply_to_all_entries_of_illust(&self, lib: &Library) -> ColEyre {
        // First, get all the entries with the url of the illust
        let url = format!("https://www.pixiv.net/en/artworks/{}", self.illust_id);

        let condition = Queryfragments::from(EqField::new(
            "URL".into(),
            FieldValue::Text(url.to_string()),
        ));

        let mut conn = lib.db.get().await?;

        let entries = condition.fetch_all(&mut conn).await?;

        for entry in entries {
            self.apply_to_entry(&mut conn, &entry).await?;
        }

        Ok(())
    }

    pub async fn download_images(
        &self,
        lib: &Library,
        overwrite_file: bool,
    ) -> ColEyreVal<Vec<Entry>> {
        let id: u64 = self.illust_id.parse()?;
        let pages = PixivProvider::fetch_illust_pages(id).await?;

        stream::iter(pages)
            .enumerate()
            .map(async |(nb, page)| {
                page.download_and_insert(lib, id, nb.try_into().unwrap(), overwrite_file)
                    .await
            })
            .buffer_unordered(1)
            .map_ok(stream::iter)
            .flatten_ok()
            .map_ok(async |entry| {
                self.apply_to_entry(&mut *lib.db.get().await?, &entry)
                    .await
                    .map(|_| entry)
            })
            .try_buffer_unordered(8)
            .try_collect_vec()
            .await
    }
}
