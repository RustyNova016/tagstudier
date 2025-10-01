use core::fmt::Display;
use core::fmt::write;

use clap::Parser;
use color_eyre::eyre::Context;
use inquire::Select;
use inquire_derive::Selectable;
use itertools::Itertools;
use rand::Rng;
use rand::seq::IndexedRandom;
use rand::seq::SliceRandom;
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
        let mut last_entry_in_chain = Self::pick_entry(&tsr, &tree, &[]).await?;
        let mut new_entry = Self::pick_entry_from_other(&tsr, &tree, &last_entry_in_chain).await?;

        loop {
            let res = Self::prompt_comp(&tsr, &last_entry_in_chain, &new_entry).await?;

            match res {
                CompRes::Same => {
                    new_entry =
                        Self::pick_entry_from_other(&tsr, &tree, &last_entry_in_chain).await?;
                }

                CompRes::AIsBetter => {
                    println!("{} better than {}", last_entry_in_chain.id, new_entry.id);
                    tree.add_rel_and_save(&tsr, last_entry_in_chain.id, new_entry.id)
                        .await?;
                    last_entry_in_chain = new_entry;
                    new_entry =
                        Self::pick_entry_from_other(&tsr, &tree, &last_entry_in_chain).await?;
                }

                CompRes::BIsBetter => {
                    println!("{} better than {}", new_entry.id, last_entry_in_chain.id);
                    tree.add_rel_and_save(&tsr, new_entry.id, last_entry_in_chain.id)
                        .await?;
                    last_entry_in_chain = new_entry;
                    new_entry =
                        Self::pick_entry_from_other(&tsr, &tree, &last_entry_in_chain).await?;
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

    async fn pick_entry_from_other(
        tsr: &TSRLibrary,
        tree: &RankTree,
        entry: &Entry,
    ) -> ColEyreVal<Entry> {
        let mut blacklist = vec![entry.id];
        blacklist.extend(tree.get_worse_entries(entry.id));
        blacklist.extend(tree.get_better_entries(entry.id));

        Self::pick_entry(tsr, tree, &blacklist).await
    }

    async fn pick_entry_id(
        tsr: &TSRLibrary,
        tree: &RankTree,
        blacklist: &[i64],
    ) -> ColEyreVal<i64> {
        let mut orders = vec![
            ImgPickingOrd::Tops,
            ImgPickingOrd::Bottoms,
            ImgPickingOrd::AnyRanked,
            ImgPickingOrd::Any,
        ];
        orders.shuffle(&mut rand::rng());

        let mut iter: Box<dyn Iterator<Item = i64>> = Box::new(Vec::new().into_iter());
        for ord in orders {
            match ord {
                ImgPickingOrd::Tops => iter = Box::new(iter.chain(tree.get_tops().map(|id| *id))),
                ImgPickingOrd::Bottoms => {
                    iter = Box::new(iter.chain(tree.get_bottoms().map(|id| *id)))
                }
                ImgPickingOrd::AnyRanked => {
                    iter = Box::new(
                        iter.chain(tree.join().left_table().iter().map(|id| *id))
                            .chain(tree.join().right_table().iter().map(|id| *id)),
                    )
                }
                ImgPickingOrd::Any => {
                    let ids = Self::get_all_entry_ids(&tsr.library).await?;
                    iter = Box::new(iter.chain(ids.into_iter()))
                }
            }
        }

        let iter = iter.filter(|id| !blacklist.contains(id));
        let candidates = iter.take(10).collect_vec();

        Ok(candidates
            .choose(&mut rand::rng())
            .expect("Couldn't find any entry ids. Is there an entry in the db?")
            .to_owned())
    }

    async fn pick_entry(tsr: &TSRLibrary, tree: &RankTree, blacklist: &[i64]) -> ColEyreVal<Entry> {
        Ok(Entry::find_by_id(
            &mut *tsr.library.db.get().await?,
            Self::pick_entry_id(tsr, tree, blacklist).await?,
        )
        .await?
        .unwrap())
    }

    async fn pick_random_new_entry(lib: &Library, blacklist: &[i64]) -> ColEyreVal<Entry> {
        let blacklist = serde_json::to_string(blacklist)?;
        Ok(sqlx::query_as!(
                    Entry,
                    "SELECT * FROM `entries` WHERE `entries`.`id` NOT IN (SELECT value FROM JSON_EACH($1)) ORDER BY RANDOM() LIMIT 1",
                    blacklist
                ).fetch_one(&mut *lib.db.get().await?).await?)
    }

    async fn get_all_entry_ids(lib: &Library) -> ColEyreVal<Vec<i64>> {
        Ok(
            sqlx::query_scalar!("SELECT id FROM `entries` ORDER BY RANDOM()")
                .fetch_all(&mut *lib.db.get().await?)
                .await?,
        )
    }

    pub async fn prompt_comp(
        tsr: &TSRLibrary,
        entry_a: &Entry,
        entry_b: &Entry,
    ) -> ColEyreVal<CompRes> {
        print!("{}[2J", 27 as char);
        let conf_a = viuer::Config {
            x: 0,
            y: 0,
            width: Some(50),
            allow_vscode: true,
            ..Default::default()
        };

        let conf_b = viuer::Config {
            x: 50,
            y: 0,
            width: Some(50),
            allow_vscode: true,
            ..Default::default()
        };

        let path_a = entry_a
            .get_global_path(&mut *tsr.library.db.get().await?)
            .await?;

        let path_b = entry_b
            .get_global_path(&mut *tsr.library.db.get().await?)
            .await?;

        viuer::print_from_file(&path_a, &conf_a)
            .expect(&format!("Image printing failed: {} ", path_a.display()));

        viuer::print_from_file(&path_b, &conf_b)
            .expect(&format!("Image printing failed: {} ", path_b.display()));

        println!("A: {}, B: {}", entry_a.id, entry_b.id);

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

pub enum ImgPickingOrd {
    Tops,
    Bottoms,
    AnyRanked,
    Any,
}
