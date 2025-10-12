use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Context;
use color_eyre::eyre::eyre;
use tagstudio_db::Entry;

use crate::ColEyre;
use crate::models::cli_utils::cli_data::CLI_DATA;
/// Merge two tags together
#[derive(Parser, Debug, Clone)]
pub struct MergeEntriesCommand {
    /// The entry to merge into
    entry: String,

    /// The entry(ies) to merge into the target
    entries_to_merge: Vec<String>,
}

impl MergeEntriesCommand {
    pub async fn run(&self) -> ColEyre {
        let lib = CLI_DATA.read().await.get_library().await?;
        let conn = &mut *lib
            .db
            .get()
            .await
            .context("Couldn't open a new connection to the library database")?; 

        let target_path = PathBuf::from(&self.entry).canonicalize()?;
        let mut target = Entry::find_by_cannon_path(conn, &target_path).await?;

        if target.is_empty() {
            return Err(eyre!("Couldn't find entry at `{}`", target_path.display()));
        } else if target.len() > 2 {
            return Err(eyre!(
                "Found multiple entries at `{}`",
                target_path.display()
            ));
        }
        let target = target.pop().unwrap();

        for entry_to_merge in &self.entries_to_merge {
            let merged_path = PathBuf::from(entry_to_merge).canonicalize()?;
            let merged = Entry::find_by_cannon_path(conn, &merged_path).await?;

            for entry in merged {
                target.merge_entry(conn, entry).await?;
            }
        }

        Ok(())
    }
}
