pub mod allow_list;
pub mod tagless_entry;
use std::sync::LazyLock;

use inquire_derive::Selectable;
use itertools::Itertools;
use sqlx::SqliteConnection;
use tagstudio_db::Entry;
use tagstudio_db::Tag;
use tagstudio_db::query::entry_search_query::EntrySearchQuery;
use tagstudio_db::query::eq_entry_id::EqEntryId;
use tagstudio_db::query::eq_tag_id::EqTagId;
use tagstudio_db::query::tag_search_query::TagSearchQuery;
use tagstudio_db::query::trait_entry_filter::QueryEntryFilter;
use tracing::warn;

use crate::cli::find_entry::allow_list::create_allow_list;
use crate::cli::find_entry::tagless_entry::ask_tagless;
use crate::interface::images::cache::IMAGE_CACHE;
use crate::interface::images::print_entry_to_cli;
use crate::models::cli_utils::cli_data::CLI_DATA;

static DISPLAY_CONF: LazyLock<viuer::Config> = LazyLock::new(|| viuer::Config {
    height: Some(55),
    x: 0,
    y: 0,
    allow_vscode: true,
    ..Default::default()
});

#[derive(clap::Parser, Debug, Clone)]
pub struct FindEntryommand {}

impl FindEntryommand {
    pub async fn run(&self) {
        let lib = CLI_DATA.read().await.get_library().await.unwrap();
        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let mut blacklisted_tags: Vec<Tag> = Vec::new();
        let mut allowed_tags: Vec<Tag> = Vec::new();
        let mut suggested_tags: Vec<Tag> = Vec::new();
        let mut blacklisted_entries = Vec::new();

        loop {
            let mut blacklist = TagSearchQuery::from(EqTagId(0));

            if let Some(filter) = get_tags_query(&blacklisted_tags) {
                blacklist = blacklist.or(filter)
            }

            let mut blacklist = blacklist.into_entry_search_query().invert();

            if let Some(filter) = get_entry_query(&blacklisted_entries) {
                blacklist = blacklist.and(filter.invert())
            }

            if let Some(filter) = create_allow_list(&allowed_tags) {
                let filter = filter.into_entry_search_query();
                blacklist = blacklist.and(filter)
            }

            println!("Searching");
            let Some(entry) = blacklist.fetch_one(conn).await.unwrap() else {
                warn!("Couldn't find any entry with those parameters. Removing some tags...");
                let first = blacklisted_tags.first().map(|tag| tag.id).unwrap_or(0);
                blacklisted_tags.retain(|tag| tag.id != first);

                let first = allowed_tags.first().map(|tag| tag.id).unwrap_or(0);
                allowed_tags.retain(|tag| tag.id != first);

                suggested_tags = Vec::new();

                continue;
            };
            println!("Entry {}", entry.id);
            blacklisted_entries.push(entry.clone());

            if !entry.path.ends_with("png") && !entry.path.ends_with("jpg") {
                warn!(
                    "Ingoring entry {} with invalid filename {}",
                    entry.id, entry.filename
                );
                continue;
            }

            IMAGE_CACHE.get_or_init(&lib, entry.id).await.unwrap();
            print!("{}[2J", 27 as char);
            print_entry_to_cli(&lib, &entry, &DISPLAY_CONF)
                .await
                .unwrap();

            println!("Entry {}", entry.id);

            let Some(tag) = select_best_question_tag(conn, &entry, &suggested_tags).await else {
                if ask_tagless() {
                    break;
                } else {
                    continue;
                }
            };

            let res = Response::select(&format!("Has tag {}?", tag.name))
                .prompt()
                .unwrap();

            suggested_tags.push(tag.clone());

            match res {
                Response::No => {
                    blacklisted_tags.push(tag);
                }
                Response::Unknown => {}
                Response::Yes => allowed_tags.push(tag),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Selectable, strum::Display)]
enum Response {
    No,
    Yes,
    Unknown,
}

fn get_tags_query(tags: &[Tag]) -> Option<TagSearchQuery> {
    let mut iter = tags.iter().map(get_tag_filter);

    let mut acc = iter.next()?;
    for right in iter {
        acc = acc.or(right)
    }

    Some(acc)
}

fn get_entry_query(entries: &[Entry]) -> Option<EntrySearchQuery> {
    let mut iter = entries
        .iter()
        .map(|entry| EntrySearchQuery::EqEntryId(EqEntryId(entry.id)));

    let mut acc = iter.next()?;
    for right in iter {
        acc = acc.or(right)
    }

    Some(acc)
}

fn get_tag_filter(tag: &Tag) -> TagSearchQuery {
    TagSearchQuery::from(EqTagId(tag.id))
}

async fn select_best_question_tag(
    conn: &mut SqliteConnection,
    entry: &Entry,
    blacklisted_tags: &[Tag],
) -> Option<Tag> {
    let mut tags = entry
        .get_tags(conn)
        .await
        .unwrap()
        .into_iter()
        .filter(|tag| !blacklisted_tags.contains(tag))
        .collect_vec();

    let mut best_tag = tags.pop()?;
    let mut best_tag_count = best_tag.get_entry_count(conn).await.unwrap();

    while let Some(tag) = tags.pop() {
        let count = tag.get_entry_count(conn).await.unwrap();
        if count > best_tag_count {
            best_tag = tag;
            best_tag_count = count;
        }
    }

    Some(best_tag)
}
