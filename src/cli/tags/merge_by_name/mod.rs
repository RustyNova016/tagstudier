use core::cmp::Reverse;

use clap::Parser;
use tagstudio_db::Tag;

use crate::models::cli_utils::cli_data::CLI_DATA;

/// Merge the tags all having the same name (or alias) provided
#[derive(Parser, Debug, Clone)]
pub struct TagsMergeByNameCommand {
    /// The tag to edit
    name: String,

    /// Do not apply the changes
    #[clap(short, long)]
    dry: bool,
}

impl TagsMergeByNameCommand {
    pub async fn run(&self) {
        let lib = CLI_DATA.read().await.get_library().await.unwrap();
        let conn = &mut *lib
            .db
            .get()
            .await
            .expect("Couldn't open a new connection to the library database");

        let mut tags = Tag::find_by_name_or_alias(conn, self.name.to_string())
            .await
            .unwrap();

        tags.sort_by_key(|t| Reverse(t.id));

        let Some(left_tag) = tags.pop() else {
            println!("No tags have been found");
            return;
        };

        while let Some(right_tag) = tags.pop() {
            let tag_id = right_tag.id;

            if !self.dry {
                left_tag.merge_tag(conn, right_tag).await.unwrap();
            }

            println!("Merged tag id {tag_id} into {}", left_tag.id);
        }
    }
}
