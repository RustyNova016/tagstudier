use std::env::current_dir;

use clap::Parser;
use tagstudio_db::models::library::Library;
use tagstudio_db::sqlx::Acquire;

use crate::utils::cli_parser::parse_tag_name;

/// Rename a tag, and add its previous name as alias
#[derive(Parser, Debug, Clone)]
pub struct RenameTagCommand {
    /// The tag to edit
    tag: String,

    /// The new name of the tag
    new_name: String,
}

impl RenameTagCommand {
    pub async fn run(&self) {
        let current_dir = current_dir().expect("Couldn't get current working directory");
        let lib = Library::try_new(current_dir.clone()).expect("Couldn't get the root library");
        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let mut tag = parse_tag_name(conn, &self.tag).await.expect("Couldn't get tag");

        tag.rename(conn, &self.new_name, false).await.expect("Couldn't rename tag");
    }
}
