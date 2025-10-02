use itertools::Itertools as _;
use sequelles::ManyToManyJoin;
use tagstudio_db::Entry;

use crate::ColEyre;
use crate::cli::ranking::rank_entries::OverWriteSelect;
use crate::models::database::entry_rank::EntryRank;
use crate::models::database::entry_rank::tree::RankTree;
use crate::models::tsr_library::TSRLibrary;
use crate::utils::iter::IteratorAdditions as _;

pub struct Ranker {
    prompts: ManyToManyJoin<i64, i64>,
    pub tree: RankTree,
    entries: Vec<i64>,
}

impl Ranker {
    pub async fn try_new(tsr: &TSRLibrary, tree: RankTree) -> ColEyre<Self> {
        let entries = sqlx::query_scalar!("SELECT id FROM `entries` ORDER BY RANDOM()") // Prerandomise the entries
            .fetch_all(&mut *tsr.library.db.get().await?)
            .await?;

        let mut this = Self {
            entries,
            prompts: Default::default(),
            tree,
        };

        let ignored = EntryRank::fetch_all_ignored(tsr).await?;
        for rank in ignored {
            this.add_prompt(rank.top_entry, rank.bottom_entry);
        }

        Ok(this)
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

        // We have a random chance to prompt a random existing entry. This allow to verify that the user actually made a correct choice
        if rand::random_range(0..5) == 0 {
            for id in self.pick_any_from_tree() {
                if id != left
                    && let Some(entry) = Entry::find_by_id(conn, id).await?
                {
                    return Ok(entry);
                }
            }
        }

        let iter = self
            .pick_above_below(left)
            .chain(self.pick_entry_from_new())
            .chain(self.pick_any_from_tree())
            .unique()
            .filter(|entry| !blacklist.contains(entry));

        for entry_id in iter {
            if let Some(entry) = Entry::find_by_id(conn, entry_id).await? {
                return Ok(entry);
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
        if self
            .tree
            .add_rel_and_save(tsr, top_entry, bottom_entry)
            .await?
        {
            return Ok(());
        }

        println!(
            "Found inconsistency. Entry `{top_entry}` cannot be above entry `{bottom_entry}` as other relations determined it was below `{bottom_entry}`",
        );
        match OverWriteSelect::select("What do you want to do?").prompt()? {
            OverWriteSelect::Skip => {}
            OverWriteSelect::Invert => {
                self.tree
                    .add_rel_and_save(tsr, bottom_entry, top_entry)
                    .await?;
            }
            OverWriteSelect::OverwriteTop => {
                self.tree.delete_all_relations_of(tsr, top_entry).await?;
                self.tree
                    .add_rel_and_save(tsr, top_entry, bottom_entry)
                    .await?;
            }
            OverWriteSelect::OverwriteBottom => {
                self.tree.delete_all_relations_of(tsr, top_entry).await?;
                self.tree
                    .add_rel_and_save(tsr, top_entry, bottom_entry)
                    .await?;
            }
        }

        Ok(())
    }

    pub fn pick_entry_from_new(&self) -> impl Iterator<Item = i64> {
        self.entries
            .iter()
            .filter(|id| {
                !self.tree.join().left_table().contain_id(id)
                    && !self.tree.join().right_table().contain_id(id)
            })
            .cloned()
            .reservoir_rand(200)
    }

    /// Pick an entry that has a rank above, below, or the same as the provided entry.
    pub fn pick_above_below(&self, id: i64) -> impl Iterator<Item = i64> {
        let rank = self.tree.get_rank(id);

        let iter = self
            .tree
            .get_entries_with_ranking(rank + 1)
            .chain(self.tree.get_entries_with_ranking(rank));

        let mut iter: Box<dyn Iterator<Item = &i64>> = Box::new(iter);

        if rank != 0 {
            iter = Box::new(iter.chain(self.tree.get_entries_with_ranking(rank - 1)))
        }

        iter.unique().reservoir_rand(200).cloned()
    }

    fn pick_any_from_tree(&self) -> impl Iterator<Item = i64> {
        self.tree.get_all_entries().cloned().reservoir_rand(2000)
    }

    pub fn pick_any_entry(&self) -> impl Iterator<Item = i64> {
        self.pick_any_from_tree()
            .chain(self.pick_entry_from_new())
            .unique()
            .reservoir_rand(200)
    }
}
