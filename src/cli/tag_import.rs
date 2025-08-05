use core::future::ready;
use std::env::current_dir;

use clap::Parser;
use futures::TryStreamExt;
use streamies::TryStreamies;
use tagstudio_db::models::library::Library;
use tracing::info;

use crate::ColEyre;
use crate::models::pixiv::PixivProvider;

#[derive(Parser, Debug, Clone)]
pub struct TagImportCommand {
    /// Do not make actual changes, and just print them out
    #[clap(short, long)]
    pub dry: bool,
}

impl TagImportCommand {
    pub async fn run(&self) -> ColEyre {
        let current_dir = current_dir().expect("Couldn't get current working directory");
        let lib = Library::try_new(current_dir.clone()).expect("Couldn't get the root library");
        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let urls = sqlx::query_scalar!(
            "SELECT DISTINCT `text_fields`.`value` FROM `text_fields` WHERE `type_key` = 'URL'"
        )
        .fetch(&mut *conn)
        .try_filter_map(|opt| ready(Ok(opt)))
        .try_collect_vec()
        .await?;

        for url in urls {
            info!("Processing url: {url}");

            let Some(id) = PixivProvider::parse_illust_id(&url) else {
                continue;
            };

            let illust = PixivProvider::fetch_illust(id.parse().unwrap()).await?;
            illust.apply_to_all_entries_of_illust(&lib).await?;
        }

        Ok(())
    }
}
