use core::str::FromStr;
use std::path::Path;
use std::path::PathBuf;

use color_eyre::eyre::Context;
use tagstudio_db::models::tag::Tag;
use tagstudio_db::sqlx::SqliteConnection;

use crate::ColEyreVal;

pub async fn parse_tag_name(conn: &mut SqliteConnection, tag: &str) -> Result<Tag, crate::Error> {
    if tag.starts_with("tag_id:") {
        let tag_id = tag.split_once(":").unwrap().1;
        let tag_id: i64 = tag_id.parse().map_err(|_| {
            crate::Error::CliInput(tag.to_string(), "Not a valid tag id".to_string())
        })?;

        let tag = Tag::find_by_id(conn, tag_id)
            .await
            .transpose()
            .ok_or_else(|| {
                crate::Error::CliInput(tag.to_string(), "Tag not found".to_string())
            })??;
        return Ok(tag);
    }

    let mut tags = Tag::find_by_exact_name(conn, tag).await?;

    if tags.len() > 1 {
        return Err(crate::Error::CliInput(
            tag.to_string(),
            "Tag is ambigous. Please use `tag_id: [id]`".to_string(),
        ));
    }

    tags.pop()
        .ok_or_else(|| crate::Error::CliInput(tag.to_string(), "Tag not found".to_string()))
}

/// Parse a cli input into a canonical pathbuf
pub fn cli_parse_path_buf(data: &str) -> ColEyreVal<PathBuf> {
    PathBuf::from_str(data).unwrap().canonicalize().context(format!("Couldn't find path `{data}`. Make sure it exists"))
}