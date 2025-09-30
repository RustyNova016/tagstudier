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
use crate::utils::cli_parser::parse_tag_name;

/// Merge two tags together
#[derive(Parser, Debug, Clone)]
pub struct ShowRankCommand;

impl ShowRankCommand {
    pub async fn run(&self) -> ColEyre {
        let tsr = CLI_DATA.read().await.get_tsr_library().await?;
        let tree = Self::init_tree(&tsr).await?;
        let mut rankings = tree.get_rankings();

        let conf = viuer::Config {
            width: Some(40),
            height: Some(30),
            x: 10,
            y: 4,
            ..Default::default()
        };

        while let Some(rank) = rankings.pop() {
            let entry = Entry::find_by_id(&mut *tsr.library.db.get().await?, rank.0.id)
                .await?
                .unwrap();

            println!("Rank #{}: {}", rank.0.layer + 1, entry.filename);

            viuer::print_from_file(
                &entry
                    .get_global_path(&mut *tsr.library.db.get().await?)
                    .await?,
                &conf,
            )
            .expect("Image printing failed.");
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
}
