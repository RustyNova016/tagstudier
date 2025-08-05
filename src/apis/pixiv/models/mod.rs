pub mod illust_tag_tag;
use crate::models::pixiv::api::illust_body::IllustBody;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IllustResponse {
    //pub error: bool,
    pub body: IllustBody,
}

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum IllustBodyEnum {
//     Body(IllustBody),
//     Error(Vec<()>),
// }

impl IllustResponse {}
