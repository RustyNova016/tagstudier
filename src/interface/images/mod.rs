use tagstudio_db::Entry;
use tagstudio_db::Library;

use crate::ColEyre;
use crate::interface::images::cache::IMAGE_CACHE;

pub mod cache;

pub async fn print_entry_to_cli(lib: &Library, entry: &Entry, conf: &viuer::Config) -> ColEyre {
    let path = entry.get_full_path(&lib.path);

    if path.exists() {
        let image = IMAGE_CACHE.get_or_init(lib, entry.id).await?;
        viuer::print(&image, conf)?;
    } else {
        println!("Entry `{}` at `{}`", entry.id, path.display());
    }

    Ok(())
}
