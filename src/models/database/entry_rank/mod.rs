use sqlx::prelude::FromRow;

use crate::ColEyreVal;
use crate::models::tsr_library::TSRLibrary;

pub mod tree;

#[derive(Debug, FromRow)]
pub struct EntryRank {
    pub id: i64,
    pub top_entry: i64,
    pub bottom_entry: i64,
    pub equal: bool,
}

impl EntryRank {
    pub async fn fetch_all(tsr: &TSRLibrary) -> ColEyreVal<Vec<Self>> {
        Ok(sqlx::query_as("SELECT * FROM `entry_ranks`")
            .fetch_all(&mut *tsr.tsr_db.get_conn().await?)
            .await?)
    }
}
