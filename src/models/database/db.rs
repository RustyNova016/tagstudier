use core::str::FromStr as _;
use core::time::Duration;
use std::path::PathBuf;

use sequelles::databases::sqlite::database::SqliteDatabase;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteJournalMode;

use crate::ColEyreVal;

pub fn create_tsr_db(lib_data_folder: PathBuf) -> ColEyreVal<SqliteDatabase> {
    let db_path = lib_data_folder.join("tsr_library.sqlite");
    let optconn = SqliteConnectOptions::from_str(db_path.to_string_lossy().as_ref())?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_millis(60000));

    let db = SqliteDatabase::builder()
        .connection_config(optconn)
        .path(db_path)
        .migrations(sqlx::migrate!("./migrations"))
        .build();

    Ok(db)
}
