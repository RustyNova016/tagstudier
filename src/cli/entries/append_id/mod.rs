use std::path::PathBuf;
use std::sync::Arc;

use tagstudio_db::Entry;
use tagstudio_db::Library;

use crate::models::cli_utils::cli_data::CLI_DATA;

/// Appends the entry id to the filename
#[derive(clap::Parser, Debug, Clone)]
pub struct EntriesAppendIdCommand {
    path: PathBuf,

    #[clap(short, long)]
    dry: bool,
}

impl EntriesAppendIdCommand {
    pub async fn run(&self) -> crate::ColEyre {
        let lib = Arc::new(CLI_DATA.read().await.get_library().await?);
        let conn = &mut *lib.db.get().await?;

        let path = self.path.strip_prefix(&lib.path)?;

        println!("Path: {}", path.display());
        let entry = Entry::find_by_path(conn, &path.to_string_lossy()).await?;

        for entry in entry {
            rename_entries(&lib, entry, self.dry).await?;
        }

        Ok(())
    }
}

async fn rename_entries(lib: &Library, mut entry: Entry, dry: bool) -> crate::ColEyre {
    let entry_path = entry.get_relative_path();

    let filename = entry_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let extension = entry_path
        .extension()
        .unwrap()
        .to_os_string()
        .to_string_lossy()
        .to_string();

    let new_filename = format!("{filename} ({}).{extension}", entry.id);

    println!(
        "Renaming entry {} to {}",
        entry_path.display(),
        &entry_path.with_file_name(&new_filename).display()
    );
    if !dry {
        entry
            .move_entry(
                &mut *lib.db.get().await?,
                &lib.path.join(&entry_path.with_file_name(new_filename)),
                &lib.path,
            )
            .await?;
    }

    Ok(())
}
