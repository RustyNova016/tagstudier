use core::future::ready;

use futures::TryStreamExt;
use serde::Deserialize;
use serde::Serialize;
use tagstudio_db::Entry;
use tagstudio_db::models::tag::Tag;
use tagstudio_db::sqlx::Acquire as _;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::models::pixiv::special_tags::PIXIV_NO_TAG_DATA_UPDATE;
use crate::models::pixiv::special_tags::PIXIV_NO_TAG_UPDATE;
use crate::models::pixiv::special_tags::PIXIV_TAG_IMPORT;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IllustTags {
    pub tags: Vec<IllustTagsTags>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IllustTagsTags {
    pub tag: String,
    pub locked: bool,
    pub deletable: bool,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub romaji: Option<String>,
    pub translation: Option<IllustTagsTagsTranslation>,
}

impl IllustTagsTags {
    pub async fn add_to_entry(&self, conn: &mut sqlx::SqliteConnection, entry: &Entry) -> ColEyre {
        let mut trans = conn.begin().await?;

        // Check if the update is authorized
        if entry.has_tag(&mut trans, PIXIV_NO_TAG_UPDATE).await? {
            return Ok(());
        }

        let tags = self.upsert_tag(&mut trans).await?;
        entry.add_tags(&mut trans, &tags).await?;

        trans.commit().await?;

        Ok(())
    }

    pub async fn upsert_tag(&self, conn: &mut sqlx::SqliteConnection) -> ColEyreVal<Vec<Tag>> {
        // Try to find the corresponding tags
        let mut tags = Vec::new();

        tags.extend(Tag::find_tag_by_name(conn, &self.tag).await?);
        if let Some(tag) = &self.romaji {
            tags.extend(Tag::find_tag_by_name(conn, tag).await?);
        }

        if let Some(tag) = &self.translation.as_ref().and_then(|t| t.en.as_ref()) {
            tags.extend(Tag::find_tag_by_name(conn, tag).await?);
        }

        if !tags.is_empty() {
            let mut trans = conn.begin().await?;
            for tag in &tags {
                self.update_tag(&mut trans, tag).await?
            }

            trans.commit().await?;
            Ok(tags)
        } else {
            Ok(vec![self.insert_tag(conn).await?])
        }
    }

    pub async fn insert_tag(&self, conn: &mut sqlx::SqliteConnection) -> ColEyreVal<Tag> {
        let new_tag = Tag {
            id: 0,
            color_namespace: None,
            color_slug: None,
            disambiguation_id: None,
            icon: None,
            is_category: false,
            name: self
                .translation
                .as_ref()
                .and_then(|t| t.en.clone())
                .or(self.romaji.clone())
                .unwrap_or(self.tag.clone()),
            shorthand: None,
        };

        let mut trans = conn.begin().await?;
        let new_tag = new_tag.insert_tag(&mut trans).await?;

        let pixiv_import_tag = Tag::get_by_name_or_insert_new(&mut *trans, PIXIV_TAG_IMPORT)
            .await
            .unwrap();

        new_tag.add_parents(&mut trans, &pixiv_import_tag).await?;

        trans.commit().await?;

        Ok(new_tag)
    }

    pub async fn update_tag(&self, conn: &mut sqlx::SqliteConnection, tag: &Tag) -> ColEyreVal<()> {
        let mut trans = conn.begin().await?;

        if tag
            .get_parents(&mut trans)
            .try_any(|tag| ready(tag.name == PIXIV_NO_TAG_DATA_UPDATE))
            .await?
        {
            trans.commit().await?;
            return Ok(());
        }

        tag.add_alias(&mut trans, &self.tag).await?;

        if let Some(val) = &self.romaji {
            tag.add_alias(&mut trans, val).await?;
        }

        if let Some(val) = &self.translation.as_ref().and_then(|t| t.en.as_ref()) {
            tag.add_alias(&mut trans, val).await?;
        }

        trans.commit().await?;

        Ok(())
    }
}

/// The translation of a tag
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IllustTagsTagsTranslation {
    pub en: Option<String>,
}
