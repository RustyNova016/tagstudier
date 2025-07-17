use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    TagstudioDb(#[from] tagstudio_db::Error)
}