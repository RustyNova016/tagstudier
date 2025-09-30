use core::fmt::Display;
use core::fmt::write;

use clap::Parser;
use color_eyre::eyre::Context;
use inquire::Select;
use inquire_derive::Selectable;
use tagstudio_db::Entry;
use tagstudio_db::Library;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::models::cli_utils::cli_data::CLI_DATA;
use crate::models::database::entry_rank::EntryRank;
use crate::models::database::entry_rank::tree::RankTree;
use crate::models::tsr_library::TSRLibrary;

/// Merge two tags together
#[derive(Parser, Debug, Clone)]
pub struct RankEntriesCommand;

impl RankEntriesCommand {
    pub async fn run(&self) -> ColEyre {
        let tsr = CLI_DATA.read().await.get_tsr_library().await?;
        let mut tree = Self::init_tree(&tsr).await?;
        let mut last_entry_in_chain = Self::pick_random_new_entry(&tsr.library, &[]).await?;
        let mut new_entry = Self::pick_random_new_entry(&tsr.library, &[]).await?;

        loop {
            let res = Self::prompt_comp(&tsr, &last_entry_in_chain, &new_entry).await?;

            match res {
                CompRes::Same => {
                    new_entry = Self::pick_random_new_entry(&tsr.library, &[]).await?;
                }

                CompRes::AIsBetter => {
                    tree.add_rel_and_save(&tsr, last_entry_in_chain.id, new_entry.id)
                        .await?;
                    last_entry_in_chain = new_entry;
                    new_entry = Self::pick_random_new_entry(&tsr.library, &[]).await?;
                }

                CompRes::BIsBetter => {
                    tree.add_rel_and_save(&tsr, new_entry.id, last_entry_in_chain.id)
                        .await?;
                    last_entry_in_chain = new_entry;
                    new_entry = Self::pick_random_new_entry(&tsr.library, &[]).await?;
                }
            }
        }

        Ok(())
    }

    async fn init_tree(tsr: &TSRLibrary) -> ColEyreVal<RankTree> {
        let ranks = EntryRank::fetch_all(tsr).await?;
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

    async fn pick_random_new_entry(lib: &Library, blacklist: &[i64]) -> ColEyreVal<Entry> {
        let blacklist = serde_json::to_string(blacklist)?;
        Ok(sqlx::query_as!(
                    Entry,
                    "SELECT * FROM `entries` WHERE `entries`.`id` NOT IN (SELECT value FROM JSON_EACH($1)) ORDER BY RANDOM() LIMIT 1",
                    blacklist
                ).fetch_one(&mut *lib.db.get().await?).await?)
    }

    pub async fn prompt_comp(
        tsr: &TSRLibrary,
        entry_a: &Entry,
        entry_b: &Entry,
    ) -> ColEyreVal<CompRes> {
        println!("Comparing {} and {}", entry_a.id, entry_b.id);

        let conf = viuer::Config {
            width: Some(40),
            height: Some(30),
            x: 10,
            y: 4,
            ..Default::default()
        };

        viuer::print_from_file(
            &entry_a
                .get_global_path(&mut *tsr.library.db.get().await?)
                .await?,
            &conf,
        )
        .expect("Image printing failed.");

        viuer::print_from_file(
            &entry_b
                .get_global_path(&mut *tsr.library.db.get().await?)
                .await?,
            &conf,
        )
        .expect("Image printing failed.");

        Ok(CompRes::select("Which is better:").prompt()?)
    }
}

#[derive(Debug, Selectable, Clone, Copy)]
pub enum CompRes {
    AIsBetter,
    BIsBetter,
    Same,
}

impl Display for CompRes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompRes::AIsBetter => write!(f, "A is better"),
            CompRes::BIsBetter => write!(f, "B is better"),
            CompRes::Same => write!(f, "Skip"),
        }
    }
}
