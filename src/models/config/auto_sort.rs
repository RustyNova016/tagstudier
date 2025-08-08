use core::future::ready;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use color_eyre::eyre::Context;
use color_eyre::eyre::Error;
use futures::StreamExt;
use futures::TryStreamExt as _;
use futures::stream;
use serde::Deserialize;
use serde::Serialize;
use streamies::TryStreamies;
use tagstudio_db::Entry;
use tagstudio_db::Library;
use tagstudio_db::models::library;
use tagstudio_db::query::Queryfragments;
use tagstudio_db::query::eq_tag::EqTag;
use tracing::instrument;
use tracing::warn;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::exts::path::PathExt;
use crate::pg_counted;
use crate::pg_inc;

#[derive(Debug, Serialize, Deserialize)]
pub struct AutosortRules {
    pub rules: Vec<AutosortRule>,
}

impl AutosortRules {
    pub fn load(lib: &Library) -> ColEyreVal<Self> {
        let path = lib.path.join(".TagStudio/autosort_config.toml");
        let mut config = File::open(path)
            .context("Couldn't open the autosort config file. Make sure it exists")?;
        let mut data = String::new();
        config
            .read_to_string(&mut data)
            .context("Couldn't read the autosort config file")?;
        toml::from_str(&data).context("Couldn't parse the autosort config file")
    }

    #[instrument(skip(lib), fields(indicatif.pb_show = tracing::field::Empty))]
    pub async fn apply(&self, lib: &Library) -> ColEyre {
        pg_counted!(self.rules.len(), "Processing rules");

        let mut black_list = Vec::new();
        for rule in &self.rules {
            let processed = rule.apply_rule(lib, &black_list).await?;
            black_list.extend(processed);
            pg_inc!();
        }

        Ok(())
    }

    async fn move_entry(&self, lib: &Library, mut entry: Entry) -> ColEyre {
        for rule in &self.rules {
            if rule.move_entry(lib, &mut entry).await? {
                return Ok(());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutosortRule {
    pub tags: Vec<String>,
    pub path: String,
}

impl AutosortRule {
    /// Check if an entry is concerned by the rule
    pub async fn check_entry(&self, lib: &Library, entry: &Entry) -> ColEyreVal<bool> {
        stream::iter(&self.tags)
            .map(|tag| Self::check_entry_tag(lib, entry, tag))
            .buffer_unordered(8)
            .try_all(|has_tag| async move { has_tag })
            .await
    }

    async fn check_entry_tag(lib: &Library, entry: &Entry, tag: &str) -> ColEyreVal<bool> {
        Ok(entry.has_tag(&mut *lib.db.get().await?, &tag).await?)
    }

    pub async fn move_entry(&self, lib: &Library, entry: &mut Entry) -> ColEyreVal<bool> {
        if !self.check_entry(lib, entry).await? {
            return Ok(false);
        }

        let target = self.target_path(lib);
        let dest = target.join(&entry.filename);

        if dest == entry.get_global_path(&mut *lib.db.get().await?).await? {
            return Ok(true);
        }

        target.create_directory_if_not_exist()?;
        match entry
            .move_file_from_canon_path(&mut *lib.db.get().await?, &dest)
            .await
        {
            Ok(_) => {}
            Err(tagstudio_db::Error::PathNotInFolder) => {
                warn!(
                    "Tried to move entry {}, to {}, which outside of its parent folder. This is not currently supported. No changes have been done",
                    entry.id,
                    dest.display()
                )
            }
            Err(tagstudio_db::Error::DestinationOccupied(to)) => {
                warn!(
                    "Tried to move entry {} to `{}`, but the destination is already occupied. No changes have been done",
                    entry.id,
                    to.display()
                )
            }
            Err(err) => Err(err).with_context(|| {
                format!("When moving the entry {} to {}", entry.id, dest.display())
            })?,
        }

        Ok(true)
    }

    /// Move the entry to this rule's path. It doesn't actually check if it the entry match this rule, but check if it can move it there nonetheless
    ///
    /// If the entry has been moved, returns true
    async fn move_entry_unchecked(&self, lib: &Library, entry: &mut Entry) -> ColEyreVal<bool> {
        let target = self.target_path(lib);
        let dest = target.join(&entry.filename);

        if dest == entry.get_global_path(&mut *lib.db.get().await?).await? {
            return Ok(true);
        }

        target.create_directory_if_not_exist()?;
        match entry
            .move_file_from_canon_path(&mut *lib.db.get().await?, &dest)
            .await
        {
            Ok(_) => {}
            Err(tagstudio_db::Error::PathNotInFolder) => {
                warn!(
                    "Tried to move entry {}, to {}, which outside of its parent folder. This is not currently supported. No changes have been done",
                    entry.id,
                    dest.display()
                )
            }
            Err(tagstudio_db::Error::DestinationOccupied(to)) => {
                warn!(
                    "Tried to move entry {} to `{}`, but the destination is already occupied. No changes have been done",
                    entry.id,
                    to.display()
                )
            }
            Err(err) => Err(err).with_context(|| {
                format!("When moving the entry {} to {}", entry.id, dest.display())
            })?,
        }

        Ok(true)
    }

    pub async fn apply_rule(&self, lib: &Library, black_list: &Vec<i64>) -> ColEyreVal<Vec<i64>> {
        let mut tags = self.tags.clone();
        let Some(mut search) = tags
            .pop()
            .map(|tag| Queryfragments::EqTag(EqTag::from(tag)))
        else {
            return Ok(Vec::new());
        };

        for tag in tags {
            search = search.and(EqTag::from(tag).into())
        }

        let sql = search.as_sql();
        let query = sqlx::query_as(&sql);
        let query = search.bind(query);

        let entries = query.fetch_all(&mut *lib.db.get().await?).await?;
        let mut entries_moved = Vec::new();

        for mut entry in entries {
            if black_list.contains(&entry.id) {
                continue;
            }

            if self.move_entry_unchecked(lib, &mut entry).await? {
                entries_moved.push(entry.id);
            }
        }

        Ok(entries_moved)
    }

    pub fn target_path(&self, lib: &Library) -> PathBuf {
        lib.path.join(&self.path)
    }
}
