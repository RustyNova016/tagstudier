use crate::cli::tags::merge_by_name::TagsMergeByNameCommand;

pub mod merge_by_name;

#[derive(clap::Parser, Debug, Clone)]
pub struct TagsCommand {
    #[command(subcommand)]
    subcommands: TagsSubcommands,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum TagsSubcommands {
    MergeByName(TagsMergeByNameCommand),
}

impl TagsCommand {
    pub async fn run(&self) {
        match &self.subcommands {
            TagsSubcommands::MergeByName(val) => val.run().await,
        }
    }
}
