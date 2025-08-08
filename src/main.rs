use clap::Parser as _;

use crate::cli::Cli;
pub(crate) use crate::error::Error;
use crate::interface::tracing::init_tracer;

pub(crate) mod apis;
pub(crate) mod cli;
pub(crate) mod constants;
pub(crate) mod error;
pub(crate) mod exts;
pub(crate) mod interface;
pub(crate) mod models;
pub(crate) mod tests;
pub(crate) mod utils;

pub(crate) type ColEyreVal<T> = color_eyre::Result<T>;
pub(crate) type ColEyre = color_eyre::Result<()>;

#[tokio::main]
async fn main() -> ColEyre {
    color_eyre::install()?;
    println!("Hello, world!");
    let cli = Cli::parse();
    let _worker_guard = init_tracer(&cli);

    cli.run().await?;

    Ok(())
}
