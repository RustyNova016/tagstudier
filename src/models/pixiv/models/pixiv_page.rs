use tagstudio_db::Entry;
use tagstudio_db::Library;
use tagstudio_db::query::entry_search_query::EntrySearchQuery;
use tagstudio_db::query::eq_entry_field::EqEntryField;
use tagstudio_db::query::eq_entry_field::FieldValue;
use tagstudio_db::query::eq_entry_name::EqEntryName;
use tagstudio_db::query::eq_tag_string::EqTagString;
use tagstudio_db::query::trait_entry_filter::EntryFilter;
use tagstudio_db::query::trait_tag_filter::TagFilter;

use crate::ColEyreVal;
use crate::models::pixiv::special_tags::PIXIV_DATA_IMPORT;

/// Represent a page of an illust
pub struct PixivPage {
    id: i64,
    page_num: i64,
}

impl PixivPage {
    pub fn get_filename(&self) -> String {
        format!("illust_{}_p{}.png", self.id, self.page_num)
    }

    pub fn get_entry_search_query(&self) -> EntrySearchQuery {
        EntrySearchQuery::from(EqEntryField {
            field_type: "URL".to_string(),
            value: FieldValue::Text(format!("https://www.pixiv.net/en/artworks/{}", self.id)),
        })
        .and(
            EqTagString(PIXIV_DATA_IMPORT.to_string())
                .into_entry_filter()
                .into(),
        )
        .and(EqEntryName(self.get_filename()).into())
    }

    /// Try to find this page's entry in the database
    ///
    /// We search by:
    /// - Has an matching URL field
    /// - Has the pixiv import tag
    /// - Correct name format
    pub async fn get_entry(&self, lib: &Library) -> ColEyreVal<Vec<Entry>> {
        let search = self.get_entry_search_query();

        Ok(search.fetch_all(&mut *lib.db.get().await?).await?)
    }
}
