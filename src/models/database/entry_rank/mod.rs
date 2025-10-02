use sqlx::prelude::FromRow;

use crate::ColEyre;
use crate::models::tsr_library::TSRLibrary;

pub mod tree;

#[derive(Debug, FromRow)]
pub struct EntryRank {
    pub id: i64,
    pub top_entry: i64,
    pub bottom_entry: i64,
    pub ignored: bool,
}

impl EntryRank {
    pub async fn fetch_all(tsr: &TSRLibrary) -> ColEyre<Vec<Self>> {
        Ok(sqlx::query_as("SELECT * FROM `entry_ranks`")
            .fetch_all(&mut *tsr.tsr_db.get_conn().await?)
            .await?)
    }

    pub async fn fetch_all_ignored(tsr: &TSRLibrary) -> ColEyre<Vec<Self>> {
        Ok(
            sqlx::query_as("SELECT * FROM `entry_ranks` WHERE `ignored` = 1")
                .fetch_all(&mut *tsr.tsr_db.get_conn().await?)
                .await?,
        )
    }

    pub async fn fetch_all_non_ignored(tsr: &TSRLibrary) -> ColEyre<Vec<Self>> {
        Ok(
            sqlx::query_as("SELECT * FROM `entry_ranks`  WHERE `ignored` = 0")
                .fetch_all(&mut *tsr.tsr_db.get_conn().await?)
                .await?,
        )
    }

    pub async fn upsert(&mut self, tsr: &TSRLibrary) -> ColEyre {
        let res = sqlx::query_as("INSERT INTO `entry_ranks` (id, top_entry, bottom_entry, ignored) VALUES (NULL, $1, $2, $3) ON CONFLICT DO UPDATE SET `entry_ranks`.`equal` = $3 RETURNING *")
            .bind(self.top_entry)
            .bind(self.bottom_entry)
            .bind(self.ignored)
            .fetch_one(&mut *tsr.tsr_db.get_conn().await?).await?;

        *self = res;

        Ok(())
    }
}
