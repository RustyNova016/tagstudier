use core::fmt::Display;
use std::sync::LazyLock;

use clap::Parser;
use inquire_derive::Selectable;
use tagstudio_db::Entry;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::cli::ranking::rank_entries::ranker::Ranker;
use crate::models::cli_utils::cli_data::CLI_DATA;
use crate::models::database::entry_rank::EntryRank;
use crate::models::database::entry_rank::tree::RankTree;
use crate::models::tsr_library::TSRLibrary;
use crate::utils::printing::print_entry_to_cli;

pub mod ranker;
/// Merge two tags together
#[derive(Parser, Debug, Clone)]
pub struct RankEntriesCommand;

impl RankEntriesCommand {
    pub async fn run(&self) -> ColEyre {
        let tsr = CLI_DATA.read().await.get_tsr_library().await?;
        let tree = Self::init_tree(&tsr).await?;
        let mut ranker = Ranker::try_new(&tsr, tree).await?;

        let left_id = ranker.pick_any_entry().next().unwrap(); //TODO: Fix empty

        let mut last_entry_in_chain = Entry::find_by_id(&mut *tsr.library.db.get().await?, left_id)
            .await?
            .unwrap();
        let mut new_entry = ranker.pick_entry(&tsr, last_entry_in_chain.id).await?;

        loop {
            let res = Self::prompt_comp(&tsr, &ranker, &last_entry_in_chain, &new_entry).await?;

            match res {
                CompRes::Same => {
                    ranker.add_prompt(last_entry_in_chain.id, new_entry.id);
                }

                CompRes::Equal => {
                    ranker.add_prompt(last_entry_in_chain.id, new_entry.id);
                    let mut ranking = EntryRank {
                        id: 0,
                        top_entry: last_entry_in_chain.id,
                        bottom_entry: new_entry.id,
                        ignored: false,
                    };

                    ranking.upsert(&tsr).await?;
                }

                CompRes::AIsBetter => {
                    println!("{} better than {}", last_entry_in_chain.id, new_entry.id);
                    ranker
                        .add_rel(&tsr, last_entry_in_chain.id, new_entry.id)
                        .await?;
                    last_entry_in_chain = new_entry;
                }

                CompRes::BIsBetter => {
                    println!("{} better than {}", new_entry.id, last_entry_in_chain.id);
                    ranker
                        .add_rel(&tsr, new_entry.id, last_entry_in_chain.id)
                        .await?;
                    last_entry_in_chain = new_entry;
                }
            }

            new_entry = ranker.pick_entry(&tsr, last_entry_in_chain.id).await?;
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

    pub async fn prompt_comp(
        tsr: &TSRLibrary,
        ranker: &Ranker,
        entry_a: &Entry,
        entry_b: &Entry,
    ) -> ColEyreVal<CompRes> {
        println!("{}[2J", 27 as char);

        println!(
            "Left: {} (Rank: {}) | Right: {} (Rank: {}) ",
            entry_a.id,
            ranker.tree.get_rank(entry_a.id) + 1,
            entry_b.id,
            ranker.tree.get_rank(entry_b.id) + 1
        );

        print_entry_to_cli(&tsr.library, &entry_a, &VIUER_CONF_L).await?;
        print_entry_to_cli(&tsr.library, &entry_b, &VIUER_CONF_R).await?;

        Ok(CompRes::select("Which is better:").prompt()?)
    }
}

#[derive(Debug, Selectable, Clone, Copy)]
pub enum CompRes {
    AIsBetter,
    BIsBetter,
    Equal,
    Same,
}

impl Display for CompRes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompRes::AIsBetter => write!(f, "A is better"),
            CompRes::BIsBetter => write!(f, "B is better"),
            CompRes::Equal => write!(f, "Ignore"),
            CompRes::Same => write!(f, "Skip"),
        }
    }
}

static VIUER_CONF_L: LazyLock<viuer::Config> = LazyLock::new(|| viuer::Config {
    x: 0,
    y: 2,
    width: Some(50),
    allow_vscode: true,
    restore_cursor: false,
    ..Default::default()
});

static VIUER_CONF_R: LazyLock<viuer::Config> = LazyLock::new(|| viuer::Config {
    x: 55,
    y: 2,
    width: Some(50),
    restore_cursor: false,
    allow_vscode: true,
    ..Default::default()
});

#[derive(Debug, Selectable, Clone, Copy)]
enum OverWriteSelect {
    OverwriteTop,
    OverwriteBottom,
    Invert,
    Skip,
}

impl Display for OverWriteSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OverWriteSelect::OverwriteTop => write!(
                f,
                "Overwrite the rankings the top entry, and add the new relation"
            ),
            OverWriteSelect::OverwriteBottom => write!(
                f,
                "Overwrite the rankings the bottom entry, and add the new relation"
            ),
            OverWriteSelect::Invert => write!(f, "Invert ranking"),
            OverWriteSelect::Skip => write!(f, "Do nothing"),
        }
    }
}
