use clap::Parser;
use color_eyre::eyre::Context;

use crate::ColEyre;
use crate::models::cli_utils::cli_data::CLI_DATA;
use crate::utils::cli_parser::parse_tag_name;

/// Merge two tags together
#[derive(Parser, Debug, Clone)]
pub struct MergeTagCommand {
    /// The tag to merge into
    tag_target: String,

    /// The tag(s) to merge into the target
    tags_to_merge: Vec<String>,
}

impl MergeTagCommand {
    pub async fn run(&self) -> ColEyre {
        let lib = CLI_DATA.read().unwrap().get_library().await?;
        let conn = &mut *lib
            .db
            .get()
            .await
            .context("Couldn't open a new connection to the library database")?;

        let target = parse_tag_name(conn, &self.tag_target).await?;

        for tag_to_merge in &self.tags_to_merge {
            let merged = parse_tag_name(conn, tag_to_merge).await?;

            target.merge_tag(conn, merged).await?;
        }

        Ok(())
    }
}
