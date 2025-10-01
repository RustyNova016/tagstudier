use core::fmt::Display;
use std::sync::LazyLock;

use clap::Parser;
use inquire_derive::Selectable;
use itertools::Itertools;
use rand::seq::SliceRandom;
use sequelles::ManyToManyJoin;
use tagstudio_db::Entry;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::models::cli_utils::cli_data::CLI_DATA;
use crate::models::database::entry_rank::EntryRank;
use crate::models::database::entry_rank::tree::RankTree;
use crate::models::tsr_library::TSRLibrary;
use crate::utils::iter::IteratorAdditions;
use crate::utils::printing::print_entry_to_cli;

/// Merge two tags together
#[derive(Parser, Debug, Clone)]
pub struct RankEntriesCommand;

impl RankEntriesCommand {
    pub async fn run(&self) -> ColEyre {
        let tsr = CLI_DATA.read().await.get_tsr_library().await?;
        let tree = Self::init_tree(&tsr).await?;
        let mut ranker = Ranker::try_new(&tsr, tree).await?;

        let left_id = ranker.pick_entry_from_edges().first().unwrap().clone(); //TODO: Fix empty

        let mut last_entry_in_chain = Entry::find_by_id(&mut *tsr.library.db.get().await?, left_id)
            .await?
            .unwrap();
        let mut new_entry = ranker.pick_entry(&tsr, last_entry_in_chain.id).await?;

        loop {
            let res = Self::prompt_comp(&tsr, &last_entry_in_chain, &new_entry).await?;

            match res {
                CompRes::Same => {
                    ranker.add_prompt(last_entry_in_chain.id, new_entry.id);
                    new_entry = ranker.pick_entry(&tsr, last_entry_in_chain.id).await?;
                }

                CompRes::AIsBetter => {
                    println!("{} better than {}", last_entry_in_chain.id, new_entry.id);
                    ranker
                        .add_rel(&tsr, last_entry_in_chain.id, new_entry.id)
                        .await?;
                    last_entry_in_chain = new_entry;
                    new_entry = ranker.pick_entry(&tsr, last_entry_in_chain.id).await?;
                }

                CompRes::BIsBetter => {
                    println!("{} better than {}", new_entry.id, last_entry_in_chain.id);
                    ranker
                        .add_rel(&tsr, new_entry.id, last_entry_in_chain.id)
                        .await?;
                    last_entry_in_chain = new_entry;
                    new_entry = ranker.pick_entry(&tsr, last_entry_in_chain.id).await?;
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

    pub async fn prompt_comp(
        tsr: &TSRLibrary,
        entry_a: &Entry,
        entry_b: &Entry,
    ) -> ColEyreVal<CompRes> {
        print!("{}[2J", 27 as char);

        print_entry_to_cli(&tsr.library, &entry_a, &VIUER_CONF_L).await?;
        print_entry_to_cli(&tsr.library, &entry_b, &VIUER_CONF_R).await?;

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

struct Ranker {
    prompts: ManyToManyJoin<i64, i64>,
    tree: RankTree,
    entries: Vec<i64>,
}

impl Ranker {
    pub async fn try_new(tsr: &TSRLibrary, tree: RankTree) -> ColEyre<Self> {
        let entries = sqlx::query_scalar!("SELECT id FROM `entries` ORDER BY RANDOM()") // Prerandomise the entries
            .fetch_all(&mut *tsr.library.db.get().await?)
            .await?;

        Ok(Self {
            entries,
            prompts: Default::default(),
            tree,
        })
    }

    pub fn add_prompt(&mut self, a: i64, b: i64) {
        self.prompts.add_relation_and_insert(a, b);
    }

    pub fn get_association_blacklist(&self, left_entry: i64) -> Vec<i64> {
        // Do not try to compare the same entry
        let mut blacklist = vec![left_entry];

        // Do not try to compare entries already compared
        blacklist.extend(self.prompts.get_associated_rights_by_id(left_entry));
        blacklist.extend(self.prompts.get_associated_lefts_by_id(left_entry));

        // Do not try to compare entries already compared
        blacklist.extend(self.tree.get_parent_entries(left_entry));
        blacklist.extend(self.tree.get_child_entries(left_entry));

        blacklist
    }

    pub async fn pick_entry(&self, tsr: &TSRLibrary, left: i64) -> ColEyre<Entry> {
        let conn = &mut *tsr.library.db.get().await?;
        let blacklist = self.get_association_blacklist(left);

        loop {
            let pick = rand::random_range(0..3);

            let mut iter: Box<dyn Iterator<Item = i64>> = Box::new(Vec::new().into_iter());

            // If the entry is part of the top, try to compare it to the other tops
            if self.tree.get_parent_entries(left).is_empty() {
                iter = Box::new(iter.then_chain(|| self.pick_top()))
            }

            let data = match pick {
                0 => self.pick_entry_from_sibling(left),
                1 => self.pick_entry_from_edges(),
                _ => self.pick_entry_from_new(),
            };

            iter = Box::new(iter.chain(data.into_iter()));
            let iter = iter.unique().filter(|entry| !blacklist.contains(entry));

            for entry_id in iter {
                if let Some(entry) = Entry::find_by_id(conn, entry_id).await? {
                    return Ok(entry);
                }
            }
        }

        panic!("TODO: Out of items")
    }

    pub async fn add_rel(
        &mut self,
        tsr: &TSRLibrary,
        top_entry: i64,
        bottom_entry: i64,
    ) -> ColEyre {
        self.tree
            .add_rel_and_save(tsr, top_entry, bottom_entry)
            .await
    }

    pub fn pick_entry_from_sibling(&self, left: i64) -> Vec<i64> {
        println!("Picking siblin");
        let mut sib: Vec<&i64> = self.tree.get_siblings(left);
        sib.shuffle(&mut rand::rng());
        sib.into_iter().cloned().collect_vec()
    }

    pub fn pick_entry_from_edges(&self) -> Vec<i64> {
        println!("Picking edge");
        let mut list = self
            .tree
            .get_tops()
            .chain(self.tree.get_bottoms())
            .unique()
            .cloned()
            .collect_vec();
        list.shuffle(&mut rand::rng());
        list
    }

    pub fn pick_entry_from_new(&self) -> Vec<i64> {
        println!("Picking new");
        let mut list = self
            .entries
            .iter()
            .filter(|id| {
                !self.tree.join().left_table().contain_id(id)
                    && !self.tree.join().right_table().contain_id(id)
            })
            .cloned()
            .collect_vec();

        list.shuffle(&mut rand::rng());
        list
    }

    pub fn pick_top(&self) -> impl Iterator<Item = i64> {
        self.tree.get_tops().reservoir_rand(50).cloned()
    }
}

static VIUER_CONF_L: LazyLock<viuer::Config> = LazyLock::new(|| viuer::Config {
    x: 0,
    y: 0,
    width: Some(50),
    allow_vscode: true,
    ..Default::default()
});

static VIUER_CONF_R: LazyLock<viuer::Config> = LazyLock::new(|| viuer::Config {
    x: 50,
    y: 0,
    width: Some(50),
    allow_vscode: true,
    ..Default::default()
});
