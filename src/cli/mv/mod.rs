use clap::Parser;
use color_eyre::eyre::Context;
use tagstudio_db::models::entry::Entry;
use tracing::info;

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
        let lib = CLI_DATA.read().unwrap().get_library().await?;
        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let from = cli_parse_path_buf(&self.from).context("Invalid `from` path")?;
        let to = cli_parse_path_buf(&self.to).context("Invalid `to` path")?;

        let entries = Entry::find_by_cannon_path(conn, &from)
            .await
            .expect("Couldn't get the corresponding entries");

        let mut i = 0;
        for mut entry in entries {
            info!("Moving file `{}`", entry.path);
            if !self.dry {
                entry
                    .move_file_from_canon_path(conn, &to)
                    .await
                    .expect("Couldn't move the file");
            }
            i += 1;
        }

        info!("Affected {i} entries");
        Ok(())
    }
}
