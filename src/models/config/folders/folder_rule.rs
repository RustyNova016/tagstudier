use std::path::PathBuf;

use color_eyre::eyre::Context as _;
use color_eyre::eyre::eyre;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Acquire;
use tagstudio_db::Entry;
use tagstudio_db::Library;
use tagstudio_db::query::Queryfragments;
use tracing::warn;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::exts::path::PathExt as _;
use crate::utils::cli_parser::parse_tag_name;

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderRule {
    path: String,

    #[serde(default)]
    sorting: String,

    #[serde(default)]
    add_tags: Vec<String>,
    // #[serde(default)]
    // remove_tags: Vec<String>,
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
        let mut search = Queryfragments::parse(&self.sorting)?;

        // Remove every item in the blacklist
        search = search.and(Queryfragments::eq_any_entry_id(black_list).into_not());

        let sql = search.as_sql();
        let query = sqlx::query_as(&sql);
        let query = search.bind(query);

        Ok(query.fetch_all(&mut *lib.db.get().await?).await?)
    }

    pub async fn tag_entries(&self, lib: &Library) -> ColEyre {
        let conn = &mut *lib.db.get().await?;
        let mut trans = conn.begin().await?;

        for tag in &self.add_tags {
            let tag = parse_tag_name(&mut *trans, &tag).await?;

            sqlx::query!(
                "
            INSERT INTO
                `tag_entries` (tag_id, entry_id)
            SELECT
                entries.id AS entry_id, $1 AS tag_id
            FROM
                entries
            WHERE
                REPLACE(`entries`.`path`, `entries`.`filename`, '') = $2",
                tag.id,
                self.path
            )
            .execute(&mut *trans)
            .await?;
        }

        Ok(())
    }
}
