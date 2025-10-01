use clap::Parser;
use clap::Subcommand;

use crate::cli::ranking::rank_entries::RankEntriesCommand;
use crate::cli::ranking::show::RankingShowCommand;

pub mod rank_entries;
pub mod show;


#[derive(Parser, Debug, Clone)]
pub struct RankingCommand {
    #[command(subcommand)]
    pub command: RankingSubcommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RankingSubcommands {
    Show(RankingShowCommand),
    Rank(RankEntriesCommand)
}

impl RankingCommand {
    pub async fn run(&self) -> crate::ColEyre {
        match &self.command {
            RankingSubcommands::Show(val) => val.run().await,
            RankingSubcommands::Rank(val) => val.run().await,
        }
    }
}

