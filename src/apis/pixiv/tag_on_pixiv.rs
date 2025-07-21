use tagstudio_db::models::tag::Tag;

pub async fn get_pixiv_import_tag(
    conn: &mut tagstudio_db::sqlx::SqliteConnection,
) -> Result<Tag, crate::Error> {
    let tags = Tag::find_tag_by_name(&mut *conn, "Pixiv Tag Import").await?;
    if let Some(tag) = tags.into_iter().next() {
        return Ok(tag);
    }

    let new_tag = Tag {
        id: 0,
        color_namespace: None,
        color_slug: None,
        disambiguation_id: None,
        icon: None,
        is_category: false,
        name: "Pixiv Tag Import".to_string(),
        shorthand: None,
    };

    Ok(new_tag.insert_tag(&mut *conn).await?)
}

pub async fn get_pixiv_data_import_tag(
    conn: &mut tagstudio_db::sqlx::SqliteConnection,
) -> Result<Tag, crate::Error> {
    let tags = Tag::find_tag_by_name(&mut *conn, "Pixiv Data Import").await?;
    if let Some(tag) = tags.into_iter().next() {
        return Ok(tag);
    }

    let new_tag = Tag {
        id: 0,
        color_namespace: None,
        color_slug: None,
        disambiguation_id: None,
        icon: None,
        is_category: false,
        name: "Pixiv Data Import".to_string(),
        shorthand: None,
    };

    Ok(new_tag.insert_tag(&mut *conn).await?)
}
