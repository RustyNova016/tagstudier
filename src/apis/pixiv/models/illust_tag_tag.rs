use serde::Deserialize;
use serde::Serialize;

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

impl IllustTagsTags {}

/// The translation of a tag
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IllustTagsTagsTranslation {
    pub en: Option<String>,
}
