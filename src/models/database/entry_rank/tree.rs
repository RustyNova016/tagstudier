use core::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashSet;

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

    pub fn is_below_entry(&self, top_entry: i64, bottom_entry: i64) -> bool {
        let mut parents = self.join.get_associated_lefts(&bottom_entry);

        while let Some(parent) = parents.pop() {
            if *parent == top_entry {
                return true;
            }

            parents.extend(self.join.get_associated_lefts(&parent));
        }

        false
    }

    pub fn get_rankings(&self) -> BinaryHeap<Reverse<TopRanking>> {
        let mut dups = HashSet::new();

        let tops = self.join.clone().into_many_to_zero_left();
        let mut items = tops
            .into_iter()
            .filter_map(|(top, bottoms)| {
                let Some(top) = top else { return None };

                if !bottoms.is_empty() {
                    return None;
                }

                Some(TopRanking { id: top, layer: 0 })
            })
            .collect_vec();

        let mut res = BinaryHeap::new();

        while let Some(item) = items.pop() {
            if dups.contains(&item.id) {
                continue;
            }
            dups.insert(item.id);

            let childrens = self.join.get_associated_lefts_by_id(item.id);
            items.extend(childrens.into_iter().map(|child| TopRanking {
                layer: item.layer + 1,
                id: *child,
            }));

            res.push(Reverse(item));
        }

        res
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
