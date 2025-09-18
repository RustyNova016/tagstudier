use core::str::FromStr as _;
use std::env::current_dir;
use std::path::PathBuf;
use core::str::FromStr as _;

use clap::Parser;
use color_eyre::eyre::Context;
use color_eyre::eyre::eyre;
use tagstudio_db::models::entry::Entry;
use tracing::error;

use crate::exts::path::PathExt;
use crate::models::cli_utils::cli_data::CLI_DATA;
use crate::utils::cli_parser::cli_parse_path_buf;

/// Move a file within the library, while keeping all the metadata attached
#[derive(Parser, Debug, Clone)]
pub struct MVCommand {
    /// The file to move
    pub from: String,

    /// Where to move it
    pub to: String,

    /// Add this flag to not make changes, and instead print them out
    #[clap(short, long)]
    pub dry: bool,
}

impl MVCommand {
    pub async fn run(&self) -> crate::ColEyre {
        let lib = CLI_DATA.read().await.get_library().await?;

        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let from = cli_parse_path_buf(&self.from).context("Invalid `from` path")?;
        let to = current_dir()?
            .join(PathBuf::from(&self.to))
            .normalize_lexically_stable()?;

        let mut entries = Entry::find_by_cannon_path(conn, &from)
            .await
            .expect("Couldn't get the corresponding entries");

        if entries.len() > 1 {
            return Err(eyre!(
                "Found multiple entries at the `from` location. Please check the database"
            ));
        }

        let Some(mut entry) = entries.pop() else {
            error!("Couldn't find any file at `{}`", from.display());
            return Ok(());
        };

        if !self.dry {
            entry
                .move_file_from_canon_path(conn, &to)
                .await
                .expect("Couldn't move the file");
        }

        Ok(())
    }
}
