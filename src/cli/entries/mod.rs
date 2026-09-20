pub mod append_id;
use crate::cli::entries::append_id::EntriesAppendIdCommand;
use crate::cli::entries::clippy::EntriesClippyCommand;

pub mod clippy;

#[derive(clap::Parser, Debug, Clone)]
pub struct EntriesCommand {
    #[command(subcommand)]
    subcommands: EntriesSubcommands,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum EntriesSubcommands {
    AppendId(EntriesAppendIdCommand),
    Clippy(EntriesClippyCommand),
}

impl EntriesCommand {
    pub async fn run(&self) -> crate::ColEyre {
        match &self.subcommands {
            EntriesSubcommands::AppendId(val) => val.run().await,
            EntriesSubcommands::Clippy(val) => val.run().await,
        }
    }
}
