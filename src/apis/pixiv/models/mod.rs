use serde::Deserialize;
use serde::Serialize;
use tagstudio_db::models::entry::Entry;
use tagstudio_db::models::tag::Tag;
use tagstudio_db::sqlx::Acquire as _;
use tagstudio_db::sqlx::SqliteConnection;

use crate::apis::pixiv::tag_on_pixiv::get_pixiv_data_import_tag;
use crate::apis::pixiv::tag_on_pixiv::get_pixiv_import_tag;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IllustResponse {
    pub body: IllustBody,
}

impl IllustResponse {
    pub async fn associate_data(&self, conn: &mut SqliteConnection, entry: &Entry) {
        let mut trans = conn.begin().await.expect("Couldn't start transaction");

        for tag in &self.body.tags.tags {
            let tags = tag
                .insert_pixiv_tag(&mut trans)
                .await
                .expect("Couldn't insert tag");

            for tag in tags {
                entry
                    .add_tag(&mut trans, tag.id)
                    .await
                    .expect("Couldn't add tag to entry");
            }
        }

        let pixiv_import_tag = get_pixiv_data_import_tag(&mut *trans).await.unwrap();
        entry
            .add_tag(&mut trans, pixiv_import_tag.id)
            .await
            .unwrap();

        trans.commit().await.expect("Couldn't commit transaction");
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IllustBody {
    title: String,
    user_name: String,
    pub tags: IllustTags,

    /// Whether the work is AI or not. 1 -> Not AI, 2 -> AI
    ai_type: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IllustTags {
    pub tags: Vec<IllustTagsTags>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub async fn find_db_tags(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Tag>, crate::Error> {
        let mut tags = Vec::new();

        tags.extend(Tag::find_tag_by_name(conn, &self.tag).await?);
        if let Some(tag) = &self.romaji {
            tags.extend(Tag::find_tag_by_name(conn, tag).await?);
        }

        if let Some(tag) = &self.translation.as_ref().and_then(|t| t.en.as_ref()) {
            tags.extend(Tag::find_tag_by_name(conn, tag).await?);
        }

        Ok(tags)
    }

    pub async fn insert_pixiv_tag(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Tag>, crate::Error> {
        // Try to find the tag in the DB
        let mut tags = self.find_db_tags(conn).await?;
        let mut trans = conn.begin().await?;

        if tags.is_empty() {
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

            tags.push(new_tag.insert_tag(&mut *trans).await?);
        }

        let pixiv_import_tag = get_pixiv_import_tag(&mut *trans).await?;

        for tag in &tags {
            tag.add_alias(&mut *trans, &self.tag).await?;

            if let Some(name) = &self.romaji {
                tag.add_alias(&mut *trans, name).await?;
            }

            if let Some(name) = &self.translation.as_ref().and_then(|t| t.en.as_ref()) {
                tag.add_alias(&mut *trans, name).await?;
            }

            tag.add_parent(&mut *trans, pixiv_import_tag.id).await?;
        }

        trans.commit().await?;

        Ok(tags)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IllustTagsTagsTranslation {
    pub en: Option<String>,
}
