use core::fmt::Display;

use clap::Parser;
use inquire_derive::Selectable;
use tagstudio_db::Entry;

use crate::utils::printing::print_entry_to_cli;
use crate::ColEyre;
use crate::ColEyreVal;
use crate::models::cli_utils::cli_data::CLI_DATA;
use crate::models::database::entry_rank::EntryRank;
use crate::models::database::entry_rank::tree::RankTree;
use crate::models::tsr_library::TSRLibrary;

/// Merge two tags together
#[derive(Parser, Debug, Clone)]
pub struct RankingShowCommand;

impl RankingShowCommand {
    pub async fn run(&self) -> ColEyre {
        let tsr = CLI_DATA.read().await.get_tsr_library().await?;
        let tree = Self::init_tree(&tsr).await?;
        let mut rankings = tree.get_rankings();

        let conf = viuer::Config {
            height: Some(40),
            x: 0,
            y: 0,
            allow_vscode: true,
            ..Default::default()
        };

        while let Some(rank) = rankings.pop() {
            let entry = Entry::find_by_id(&mut *tsr.library.db.get().await?, rank.0.id)
                .await?
                .unwrap();

            print!("{}[2J", 27 as char);
            print_entry_to_cli(&tsr.library, &entry, &conf).await?;

            println!(
                "Rank #{}: {} ({})",
                rank.0.layer + 1,
                entry.filename,
                entry.id
            );

            match UserAction::select("").prompt()? {
                UserAction::Continue => {}
                UserAction::Exit => break,
                UserAction::Favourite => {
                    entry
                        .add_tag_id(&mut *tsr.library.db.get().await?, 1) // Favourite is tag ID 1
                        .await?
                }
            }
        }

        Ok(())
    }

    async fn init_tree(tsr: &TSRLibrary) -> ColEyreVal<RankTree> {
        let ranks = EntryRank::fetch_all_non_ignored(tsr).await?;
        let mut tree = RankTree::new();

        for rank in ranks {
            if !tree.add_rel(rank.top_entry, rank.bottom_entry) {
                panic!(
                    "Found loop in db data: Cannot add {} and {}",
                    rank.top_entry, rank.bottom_entry
                )
            }
        }

        Ok(tree)
    }
}

#[derive(Debug, Selectable, Clone, Copy)]
pub enum UserAction {
    Continue,
    Favourite,
    Exit,
}

impl Display for UserAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserAction::Continue => write!(f, "Continue"),
            UserAction::Favourite => write!(f, "Favourite"),
            UserAction::Exit => write!(f, "Exit"),
        }
    }
}
