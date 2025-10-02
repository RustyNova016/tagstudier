use color_eyre::eyre::Ok;
use tagstudio_db::Entry;
use tagstudio_db::Library;

use crate::ColEyre;
use crate::models::caching::image_cache::IMAGE_CACHE;

pub async fn print_entry_to_cli(lib: &Library, entry: &Entry, conf: &viuer::Config) -> ColEyre {
    let path = entry.get_global_path(&mut *lib.db.get().await?).await?;

    if path.exists() {
        let image = IMAGE_CACHE.get_or_init(lib, entry.id).await?;
        viuer::print(&image, &conf)?;
    } else {
        println!("Entry `{}` at `{}`", entry.id, path.display());
    }

    Ok(())
}
