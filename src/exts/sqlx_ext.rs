use sqlx::Acquire;
use sqlx::SqliteConnection;
use sqlx::Transaction;

pub trait AcquireWrite<'a>: sqlx::Acquire<'a> {
    async fn begin_write(
        self,
    ) -> Result<Transaction<'a, <Self as Acquire<'a>>::Database>, sqlx::Error>;
}

impl<'a> AcquireWrite<'a> for &'a mut SqliteConnection {
    async fn begin_write(
        self,
    ) -> Result<Transaction<'a, <Self as Acquire<'a>>::Database>, sqlx::Error> {
        let mut trans = self.begin().await?;
        sqlx::query("UPDATE `preferences` SET `value` = '' WHERE `key` = 'qhdopqojfjpojepjfpos'")
            .execute(&mut *trans)
            .await?;

        Ok(trans)
    }
}
