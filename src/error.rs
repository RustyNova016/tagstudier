use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    TagstudioDb(#[from] tagstudio_db::Error),

    #[error(transparent)]
    LibraryConnection(#[from] tagstudio_db::TSPoolError),

        #[error(transparent)]
    Sqlx(#[from] tagstudio_db::sqlx::Error),

}
