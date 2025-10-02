use sequelles::databases::sqlite::database::SqliteDatabase;
use sqlx::Acquire;
use tagstudio_db::Library;

use crate::ColEyre;
use crate::ColEyreVal;
use crate::models::database::db::create_tsr_db;

pub struct TSRLibrary {
    pub library: Library,
    pub tsr_db: SqliteDatabase,
}

impl TSRLibrary {
    pub fn try_new(lib: Library) -> ColEyreVal<TSRLibrary> {
        let db = create_tsr_db(lib.path.join(".TagStudio"))?;

        Ok(Self {
            tsr_db: db,
            library: lib,
        })
    }

    /// Synchronize the data between the two dbs
    pub async fn sync(&self) -> ColEyre {
        let entry_ids = sqlx::query_scalar!("SELECT `id` FROM `entries`")
            .fetch_all(&mut *self.library.db.get().await?)
            .await?;
        let entry_ids = serde_json::to_string(&entry_ids)?;

        let conn = &mut *self.tsr_db.get_conn().await?;
        let mut trans = conn.begin().await?;

        sqlx::query("INSERT INTO `entries` SELECT value as id FROM JSON_EACH($1)")
            .bind(&entry_ids)
            .execute(&mut *trans)
            .await?;

        sqlx::query("DELETE FROM `entries` WHERE `id` NO IN (SELECT value FROM JSON_EACH($1))")
            .bind(entry_ids)
            .execute(&mut *trans)
            .await?;

        trans.commit().await?;

        Ok(())
    }
}
