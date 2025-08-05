use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IllustUrls {
    pub thumb_mini: Option<String>,
    pub small: Option<String>,
    pub regular: Option<String>,
    pub original: Option<String>,
}
