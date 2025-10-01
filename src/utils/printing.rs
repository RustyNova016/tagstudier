use color_eyre::eyre::Ok;
use tagstudio_db::Entry;
use tagstudio_db::Library;

use crate::ColEyre;

pub async fn print_entry_to_cli(lib: &Library, entry: &Entry, conf: &viuer::Config) -> ColEyre {
    let path = entry.get_global_path(&mut *lib.db.get().await?).await?;

    if path.exists() {
        viuer::print_from_file(&path, &conf)?;
    } else {
        println!("Entry `{}` at `{}`", entry.id, path.display());
    }

    Ok(())
}
