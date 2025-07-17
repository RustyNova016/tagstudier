use std::env::current_dir;
use std::path::PathBuf;

use clap::Parser;
use tagstudio_db::models::entry::Entry;
use tagstudio_db::models::library::Library;
use tracing::info;

/// Move a file within the library, while keeping all the metadata attached
#[derive(Parser, Debug, Clone)]
pub struct MVCommand {
    /// The file to move
    pub from: PathBuf,

    /// Where to move it
    pub to: PathBuf,

    /// Add this flag to not make changes, and instead print them out
    #[clap(short, long)]
    pub dry: bool,
}

impl MVCommand {
    pub async fn run(&self) {
        let current_dir = current_dir().expect("Couldn't get current working directory");
        let lib = Library::try_new(current_dir.clone()).expect("Couldn't get the root library");
        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let from = self.from.canonicalize().expect("Invalid `from` path");
        let to = self.to.canonicalize().expect("Invalid `to` path");

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

        info!("Affected {i} entries")
    }
}
