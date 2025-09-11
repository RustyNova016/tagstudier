use std::path::PathBuf;

use color_eyre::eyre::Context as _;
use color_eyre::eyre::eyre;
use serde::Deserialize;
use serde::Serialize;
use tagstudio_db::Entry;
use tagstudio_db::Library;
use tagstudio_db::query::entry_search_query::EntrySearchQuery;
use tagstudio_db::query::eq_any_entry_id::EqAnyEntryId;
use tagstudio_db::query::trait_entry_filter::EntryFilter as _;
use tracing::warn;

use crate::ColEyreVal;
use crate::exts::path::PathExt as _;

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderRule {
    path: String,

    #[serde(default)]
    sorting: String,
}

impl FolderRule {
    pub fn absolute_path(&self, lib: &Library) -> PathBuf {
        lib.path.join(&self.path)
    }

    pub async fn sort_entries(&self, lib: &Library, black_list: Vec<i64>) -> ColEyreVal<Vec<i64>> {
        let entries = self.get_entries_to_sort(lib, black_list).await?;
        let mut entries_moved = Vec::new();

        for mut entry in entries {
            if self.move_entry_unchecked(lib, &mut entry).await? {
                entries_moved.push(entry.id);
            }
        }

        Ok(entries_moved)
    }

    /// Move the entry to this rule's path. It doesn't actually check if it the entry match this rule, but check if it can move it there nonetheless
    ///
    /// If the entry has been moved, returns true
    async fn move_entry_unchecked(&self, lib: &Library, entry: &mut Entry) -> ColEyreVal<bool> {
        let Some(filename) = entry.get_filename() else {
            return Err(eyre!(
                "Found an entry without a filename: {}",
                entry.path.to_string()
            ));
        };

        let dest = self.absolute_path(lib).join(filename);

        // Check if the entry isn't already there
        if dest == entry.get_global_path(&mut *lib.db.get().await?).await? {
            return Ok(true);
        }

        self.absolute_path(lib).create_directory_if_not_exist()?;
        match entry
            .move_file_from_canon_path(&mut *lib.db.get().await?, &dest)
            .await
        {
            Ok(_) => {}
            Err(tagstudio_db::Error::PathNotInFolder) => {
                warn!(
                    "Tried to move entry {}, to {}, which outside of the library folder. This is not currently supported. No changes have been done",
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

    /// Return all the entries that require sorting
    async fn get_entries_to_sort(
        &self,
        lib: &Library,
        black_list: Vec<i64>,
    ) -> ColEyreVal<Vec<Entry>> {
        let mut search = EntrySearchQuery::parse(&self.sorting)?;

        // Remove every item in the blacklist
        search = search.and(
            EntrySearchQuery::from(EqAnyEntryId(black_list))
                .invert()
                .into(),
        );

        Ok(search.fetch_all(&mut *lib.db.get().await?).await?)
    }
}
