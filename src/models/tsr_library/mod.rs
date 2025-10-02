use sequelles::databases::sqlite::database::SqliteDatabase;
use sqlx::QueryBuilder;
use sqlx::Sqlite;
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

    pub async fn sync(&self) -> ColEyre {
        let entry_ids = sqlx::query_scalar!("SELECT `id` FROM `entries`")
            .fetch_all(&mut *self.library.db.get().await?)
            .await?;

        // TODO: use json to fill data
        let mut insert: QueryBuilder<Sqlite> = QueryBuilder::new("INSERT OR IGNORE INTO `entries` (`id`) ");
        insert.push_values(entry_ids.iter(), |mut b, id| {
            b.push_bind(id);
        });

        let insert = insert.build();
        insert.execute(&mut *self.tsr_db.get_conn().await?).await?;

        //TODO: Delete Entries

        Ok(())
    }
}
