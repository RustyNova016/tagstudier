use std::env::current_dir;

use clap::Parser;
use streamies::TryStreamies as _;
use tagstudio_db::models::entry::Entry;
use tagstudio_db::models::library::Library;
use crate::apis::pixiv::fetch_illust_data;

/// Add links to images based on their filename
#[derive(Parser, Debug, Clone)]
pub struct TagImportCommand {
    /// Do not make actual changes, and just print them out
    #[clap(short, long)]
    pub dry: bool,
}

impl TagImportCommand {
    pub async fn run(&self) {
        let current_dir = current_dir().expect("Couldn't get current working directory");
        let lib = Library::try_new(current_dir.clone()).expect("Couldn't get the root library");
        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let entries = Entry::stream_entries(conn)
            .try_collect_vec()
            .await
            .expect("Couldn't get the entries");

        for entry in entries {
            let fields = entry
                .get_text_fields(conn)
                .await
                .expect("Couldn't get entry feilds");
            let Some(url) = fields.iter().find(|f| f.type_key == "URL") else {
                continue;
            };

            Self::import_entry_tags(&lib, &entry, url.value.as_ref().unwrap_or(&"".to_string()))
                .await;
        }
    }

    pub async fn import_entry_tags(lib: &Library, entry: &Entry, url: &str) {
        if !url.starts_with("https://www.pixiv.net") {
            return;
        }
        let Some(illust_id) = url.split("/").last() else {
            return;
        };
        let illust_data = fetch_illust_data(illust_id).await;

        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        illust_data.associate_data(conn, entry).await;
    }
}