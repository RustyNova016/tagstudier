use std::env::current_dir;

use clap::Parser;
use tagstudio_db::models::library::Library;

use crate::utils::cli_parser::parse_tag_name;

/// Merge two tags together
#[derive(Parser, Debug, Clone)]
pub struct MergeTagCommand {
    /// The tag to merge into
    tag_target: String,

    /// The tag to merge into the target
    tag_to_merge: String,
}

impl MergeTagCommand {
    pub async fn run(&self) {
        let current_dir = current_dir().expect("Couldn't get current working directory");
        let lib = Library::try_new(current_dir.clone()).expect("Couldn't get the root library");
        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let target = parse_tag_name(conn, &self.tag_target).await.unwrap();
        let merged = parse_tag_name(conn, &self.tag_to_merge).await.unwrap();

        target.merge_tag(conn, merged).await.unwrap();
    }
}
