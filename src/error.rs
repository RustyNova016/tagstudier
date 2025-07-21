use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    TagstudioDb(#[from] tagstudio_db::Error),

    #[error(transparent)]
    LibraryConnection(#[from] tagstudio_db::TSPoolError),

    #[error(transparent)]
    Sqlx(#[from] tagstudio_db::sqlx::Error),

    #[error("Incorrect input for CLI: {0}")]
    CliInput(String, String),
}

// pub trait CliResult {
//     fn unwrap_parse_or_exit(self) -> Self;
// }

// impl<T> CliResult for Result<T, crate::Error> {
//     fn unwrap_parse_or_exit(self) -> Self {
//         match self {
//             Self::Err(Error::CliInput(data, msg)) => {
//                 error!("Invalid input: `{data}`: {msg}");
//                 std::process::exit(exit_code);
//             }
//             Self::Ok(val) => Ok(val),
//             Self::Err(val) => Err(val),
//         }
//     }
// }
