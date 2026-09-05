use tagstudio_db::Tag;
use tagstudio_db::query::eq_tag_id::EqTagId;
use tagstudio_db::query::tag_search_query::TagSearchQuery;

pub(super) fn create_allow_list(tags: &[Tag]) -> Option<TagSearchQuery> {
    let mut iter = tags.iter().map(|tag| TagSearchQuery::from(EqTagId(tag.id)));

    let mut acc = iter.next()?;
    for right in iter {
        acc = acc.or(right)
    }

    Some(acc)
}
