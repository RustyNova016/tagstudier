use core::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::os::unix::process::parent_id;

use itertools::Itertools;
use sequelles::ManyToManyJoin;

use crate::ColEyre;
use crate::models::tsr_library::TSRLibrary;

pub struct RankTree {
    join: ManyToManyJoin<i64, i64>,
}

impl RankTree {
    pub fn new() -> Self {
        Self {
            join: ManyToManyJoin::default(),
        }
    }

    pub fn join(&self) -> &ManyToManyJoin<i64, i64> {
        &self.join
    }

    pub fn add_rel(&mut self, top_entry: i64, bottom_entry: i64) -> bool {
        // We check if the top entry isn't already set as a child for the bottom entry
        if self.is_below_entry(bottom_entry, top_entry) {
            return false;
        }

        self.join.add_relation_and_insert(top_entry, bottom_entry);
        true
    }

    pub async fn save_rel(tsr: &TSRLibrary, top_entry: i64, bottom_entry: i64) -> ColEyre {
        sqlx::query("INSERT INTO `entry_ranks` (id, top_entry, bottom_entry, equal) VALUES (NULL, $1, $2, 0)").bind(top_entry).bind(bottom_entry).execute(&mut *tsr.tsr_db.get_conn().await?).await?;
        Ok(())
    }

    pub async fn add_rel_and_save(
        &mut self,
        tsr: &TSRLibrary,
        top_entry: i64,
        bottom_entry: i64,
    ) -> ColEyre {
        if self.add_rel(top_entry, bottom_entry) {
            Self::save_rel(&tsr, top_entry, bottom_entry).await?;
        }

        Ok(())
    }

    /// Return true if the entry is below the top entry in the tree, recursively
    ///
    /// ```
    /// let tree = RankTree::new();
    /// tree.add_rel(1, 2);
    /// assert!(tree.is_below_entry(2))
    /// ```
    pub fn is_below_entry(&self, top_entry: i64, bottom_entry: i64) -> bool {
        let mut parents = self.join.get_associated_lefts(&bottom_entry);

        while let Some(parent) = parents.pop() {
            println!("par pop");
            if *parent == top_entry {
                return true;
            }

            parents.extend(self.join.get_associated_lefts(&parent));
        }

        false
    }

    pub fn get_tops(&self) -> impl Iterator<Item = &i64> {
        self.join
            .left_table()
            .iter()
            .filter(|id| self.join.get_associated_lefts_by_id(**id).is_empty())
    }

    pub fn get_bottoms(&self) -> impl Iterator<Item = &i64> {
        self.join
            .right_table()
            .iter()
            .filter(|id| self.join.get_associated_rights_by_id(**id).is_empty())
    }

    pub fn get_rankings(&self) -> BinaryHeap<Reverse<TopRanking>> {
        let mut dups = HashSet::new();

        let mut items = self
            .get_tops()
            .map(|id| TopRanking { id: *id, layer: 0 })
            .collect_vec();

        let mut res = BinaryHeap::new();

        while let Some(item) = items.pop() {
            if dups.contains(&item.id) {
                continue;
            }
            dups.insert(item.id);

            let childrens = self
                .join
                .get_associated_rights_by_id(item.id)
                .into_iter()
                .filter(|new_id| {
                    // all the parents must have been yielded before that one
                    let parents = self.get_parent_entries(**new_id);
                    parents.iter().all(|parent_id| dups.contains(&parent_id))
                });
            items.extend(childrens.into_iter().map(|child| TopRanking {
                layer: item.layer + 1,
                id: *child,
            }));

            res.push(Reverse(item));
        }

        res
    }

    pub fn get_parent_entries(&self, id: i64) -> Vec<&i64> {
        self.join.get_associated_lefts_by_id(id)
    }

    /*     pub fn get_better_entries_recursive(&self, id: i64) -> Vec<&i64> {
        let better = self.get_better_entries(id);

        if better.is_empty() {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(better.len());
        for id in better {
            out.push(id);
            out.extend(self.get_better_entries_recursive(id));
        }

    } */

    pub fn get_child_entries(&self, id: i64) -> Vec<&i64> {
        self.join.get_associated_rights_by_id(id)
    }

    pub fn get_siblings(&self, id: i64) -> Vec<&i64> {
        let mut siblings = Vec::new();

        // Get the siblings from the parents
        for parent in self.get_parent_entries(id) {
            siblings.extend(self.get_child_entries(*parent));
        }

        // Get the siblings from the childrens
        for child in self.get_parent_entries(id) {
            siblings.extend(self.get_child_entries(*child));
        }

        siblings.into_iter().unique().collect_vec()
    }

    pub fn get_rank(&self, id: i64) -> u64 {
        self.get_parent_entries(id)
            .into_iter()
            .map(|parent| self.get_rank(*parent))
            .max()
            .map(|parent_rank| parent_rank + 1)
            .unwrap_or(0)
    }

    /// Give the parent entries that are directly 1 rank above the one provided
    pub fn get_direct_parents(&self, id: i64) -> Vec<&i64> {
        let rank = self.get_rank(id);

        self.get_parent_entries(id)
            .into_iter()
            .filter(|parent| self.get_rank(**parent) + 1 == rank)
            .collect_vec()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopRanking {
    pub layer: i64,
    pub id: i64,
}

impl PartialOrd for TopRanking {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TopRanking {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.layer
            .cmp(&other.layer)
            .then_with(|| self.id.cmp(&other.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_below() {
        let mut tree = RankTree::new();
        tree.add_rel(1, 2);
        assert!(tree.is_below_entry(1, 2));

        tree.add_rel(2, 3);
        assert!(tree.is_below_entry(1, 3))
    }

    #[test]
    fn test_add_rel() {
        let mut tree = RankTree::new();
        assert!(tree.add_rel(1, 2));
        assert!(tree.add_rel(2, 3));
        assert!(!tree.add_rel(3, 1));
    }
}
