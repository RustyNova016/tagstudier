use color_eyre::eyre::eyre;
use serde::Deserialize;
use serde::Serialize;

use crate::ColEyreVal;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PixivResponse<T> {
    pub error: bool,
    pub message: String,
    body: PixivResponseBodyEnum<T>,
}

impl<T> PixivResponse<T> {
    pub fn body(&self) -> ColEyreVal<&PixivResponseBodyEnum<T>> {
        if self.error {
            return Err(eyre!("Pixiv api returned an error: {}", self.message));
        }

        Ok(&self.body)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PixivResponseBodyEnum<T> {
    Object(T),
    Array(Vec<T>),
}

impl<T> PixivResponseBodyEnum<T> {
    pub fn as_object(&self) -> Option<&T> {
        match self {
            Self::Object(val) => Some(val),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<T>> {
        match self {
            Self::Array(val) => Some(val),
            _ => None,
        }
    }
}
